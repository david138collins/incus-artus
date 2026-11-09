# The dyno: the speed checklist

Every idea queued for making sight fast, in one place. Builder ticks each one off as she goes and writes its
measured result beside it. Started 2026-11-09 at David's asking ("i really hope we dont forget any of all the good
ideas we have had for going from zero to sixty").

The why and David's words for each idea are in `docs/decisions.md`, "Look first, then the dyno". The baseline and
build-by-build numbers are in `docs/regroup.md`.

**How to use it:**
- `[ ]` not started, `[~]` tried or in progress, `[x]` kept, `[-]` tried and dropped (say why).
- Write the build number and the measured change beside each item, for example "build 101: sight 12.1 → 9.4 ms on
  the wide flight, baked, defaults".
- **Block** items must leave the picture unchanged: an A/B picture test on the card, the same frame before and
  after, equal to the pixel or within a stated tolerance.
- **Induction** items trade something for speed, so they get David's eye instead.
- One step a build, each measured alone.

**Where we start** (the baseline after build 92, the low slow flight, uncapped):
- **Defaults, baked:** sight 12 to 16.6 ms, 56 to 75 fps.
- **David's lean preset (8 × 8 and 4 × 4, no shadows), baked:** sight 3.4 to 4.7 ms, 197 to 262 fps.
- About 8 ms of the gap is shadows, sky light and the 2 × 2 layer. Builds 93 and 94 added 2 to 3 ms for far
  ground drawn as ground.

## 0. Done before the dyno

- [x] The baseline (after build 92).
- [x] The two flaws: the close-up disc and the circles from altitude (builds 93 and 94).
- [x] More storage slots asked from the card (build 95): David's card reports 262,144, and we use 9.
- [x] Nsight Graphics installed on David's machine (2026-11-09).

## 1. Instruments: measure first

- [ ] A wider fixed flight that takes in sea, mountains and a cave mouth, measured baked and baking separately.
- [ ] A performance gate: every dyno build flies the same route, with its numbers kept in `regroup.md`.
- [ ] An input-to-photon gate: the mouse stays near build 67's 1.05 ms (uncapped).
- [ ] Nsight: card time per pass, stall reasons (memory against divergence), and occupancy.
- [ ] A slowest-ray view: each tile's slowest ray against its average, built from the steps view (`reach.z`).
- [ ] Register pressure in `march.wgsl` (one big shader).
- [ ] The subgroup check: does the adapter offer them? Keep a path that works without them.
- [ ] Break down builds 93 and 94's 2 to 3 ms: shadows, sky cones or the finer layers' extra pixels.

## 2. The block: the same picture, faster

**Skipping empty air**
- [ ] Rays end where fog is total (99% fog by about 5,000 blocks, against a reach of 150,000). Likely the cheapest
      win.
- [ ] Relaxed cone stepping: each column keeps the widest empty cone over its top, so grazing rays take long safe
      steps.
- [ ] An empty-space distance per cell: the same idea in 3D, for caves and overhangs.
- [ ] Solved jumps: a ray drops straight to the height of the tallest column top below it.
- [ ] The sea as a flat plane: its top found in one sum (the 610-step horizon case). Designed with generic
      transparency.
      Build 98's seeing through water added about 1.5 ms on the low flight (rays go on to the bottom).
- [ ] The rasteriser starts the rays: boxes drawn around occupied bricks give each pixel its first possible hit.
- [ ] A two-level start pass: 64 × 64 tiles, then 8 × 8.

**Light computed once, not every frame**
- [ ] Horizon maps: far sun shadows and sky light become lookups (made at bake time, patched on edits). Same picture
      within a stated tolerance.
- [ ] Light the hologram: sun and sky light kept in the cells, re-lit gradually as the sun moves.

**Memory**
- [ ] Hot and cold data apart: solid shares in their own tight buffer, colour and faces fetched only at a hit.
- [ ] Texture units: cells as bricks in a 3D texture, blended by hardware and read through its neighbour cache.
- [ ] The store's layout: smaller cells, and neighbours side by side in memory (Morton order).
- [ ] Read-only bindings where the march only reads, and small constants as uniforms.
- [ ] f16 colour sums to free registers, within a stated rounding tolerance.

**Keeping the card's workers busy**
- [ ] Tiles by type (sky, far, near, water, seam), each with its own lean shader. Today the finer layers launch over
      the whole screen.
- [ ] Warps kept alike: rays sorted by expected length, or compaction (march N steps, pack the ones still going,
      repeat).
- [ ] No idle lanes: persistent threads, a finished lane taking the next ray from a queue (dynamic ray fetch, Aila
      and Laine 2009).
- [ ] Subgroups: warp votes for shared strides, the warp's nearest start.
- [ ] Workgroup shape and order: 8 × 8 against 16 × 4, Morton order.
- [ ] Background work, such as the bake, in the card's idle gaps between frames.
- [ ] Textures' last ~1 ms (build 87).
- [ ] Placed light under many lights (build 95): one table probe a face met near the eye (four with the blend
      across faces); check a lit cave full of braziers, and the table's fill (65,536 places, half used at most).

## 3. Induction: trades something, judged by David's eye

- [ ] Temporal reuse (TRAA), after the *INSIDE* talk: jitter (Halton 2,3), reprojection, neighbourhood clipping,
      about 90% feedback, motion blur for fast pixels (output only), noise soaked up so fewer sky cones and shadow
      samples are needed. David's rules:
      - movers never take from history;
      - pixels with no history get rays first;
      - edits and light changes drop history;
      - near hard faces stay crisp.
- [ ] Rays start just short of last frame's hits (with a back-off and the nearest of the neighbours).
- [ ] Far pixels re-marched only once they've moved half a pixel, with marching decoupled from shading (hit position
      and facing kept).
- [ ] A slowly refreshed far panorama for turning.
- [ ] A checkerboarded coarse layer, filled in over time (Fork E's remnant).
- [ ] Fixed budgets each frame: holes first, then refreshing old pixels, coarse first and sharpened after.
- [ ] Blue-noise jitter for the temporal phases.

## 4. The processor as a partner

- [ ] One-way handoffs: the processor prepares frame N + 1's data while the card draws frame N. Never a round trip;
      anything read back is read a few frames late.
- [ ] Camera prediction: the bake fills the ground the camera is heading into first.
- [ ] The turn sent ahead: the strip about to come on screen gets its rays reserved.
- [ ] Late latching and timewarp: the newest mouse read just before the frame goes, and again just before it's
      shown.
- [ ] Watch the laptop's shared heat budget: processor work can lower the card's clocks.
- [ ] The objects' frame work (build 95): every placed object in the loaded chunks gathered each frame, its
      column scanned up to the world's top for cover, its placings sent to the card afresh. Fine for a few
      braziers; check with a settlement's worth (kept between frames, redone only when objects or blocks change).
- [ ] The placed lights' flood (build 95): every light flooded again when the loaded chunks change; with many
      lights, only those near the chunks that came or went.

## 5. The first impression

- [ ] The bake's share of the card revisited (build 84 traded frame rate for bake speed).
- [ ] The bake hidden in the opening (docs/vision.md, The opening): the world forming before your eyes, and the
      first morning's fog following the bake.

## Ruled out

Tensor cores, ray-tracing cores, DLSS, Nvidia's ray sorting (SER) and hardware variable-rate shading, under David's
rule: "im not interested in implementing all the different tricks individual modern cards can use, id have to own 20
gpus".

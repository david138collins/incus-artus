# Dyno audit, 2026-11-10

An outside read of sight's speed at build 104 (commit 3f0dfc3, with the uncommitted edits to `docs/dyno.md`).
Read only: nothing was built, run or measured. **Every win below is a guess from reading the code against the
measured facts**, with a confidence beside it. Phase 1 (the code alone) was written down before `docs/` was
opened; the statuses were added after.

## 1. For David

1. The light on each block face near you is worked out afresh every frame, though it only changes when the sun
   moves or blocks change. Keeping it from one frame to the next could be the biggest single saving left.
2. As you fly, each of the five near levels works out the same ground separately. Working it out once and
   sharing it would about halve that job, with nothing changed on screen.
3. The blend where one level of detail hands over to the next runs over nearly half of all distances. A
   narrower hand-over is cheap to try; you'd judge whether rings come back over flat water.
4. The plan's next step (near walk and far march as separate passes) may free less room than hoped: by my count
   the far march's room is its own. Builder's sizing probes will say; I'd wait for them before building it.
5. The texture-unit rework is the right big bet, but it can be priced with a stand-in in one flight first, and
   its picture won't match to the pixel (the hardware blends in 256 steps).

## 2. The leads

Shares are of the card's time. "Land" means the over-the-land trace. Confidence: low, medium, high.

| # | Lead | Kind | Status | Guessed win (confidence) | The flight that prices it |
|---|---|---|---|---|---|
| 1 | Light per face kept between frames | same once settled; lags changes a few frames | **new** (kin to a dropped step) | 8 to 14% land, 4 to 7% sea (medium) | twin file: one face in eight lit a frame |
| 2 | Ground columns shared by the live levels | same picture | **new** | 5 to 8% land while flying (medium) | none: an old flight's log, by level |
| 3 | A narrower hand-over between levels | trade | **new** (the probe is theirs) | 4 to 5% (medium), if David takes it | twin file: `BLEND_FROM` 0.8 |
| 4 | Shadow and sky rays remember a failed skip | same picture | **new** | 3 to 8% land (low) | twin file |
| 5 | A fine pixel takes its parents' shadow where they agree | trade | **new** | 4 to 9% land (low) | twin file |
| 6 | `live_slot` without whole-number remainders | same picture | **new** | 1 to 4% (low) | twin file (float floor) |
| 7 | The far loop's room: `before` keeps one number, not six | same picture | **new** detail of a planned aim | 1 to 3% (low) | twin file, registers read in Nsight |
| 8 | `starts` and other read-only data bound read-only | same picture | **planned** (Memory), reason new | 0 to 3% (low) | twin can't; build against build |
| 9 | Sky pixels finished in the march; faces' pixels and cone pixels in lists of their own | same picture | **planned** in part (tiles by type) | 1 to 4% (low) | an instrument first (section 5) |
| 10 | The listing as the march's tail | same picture | **new** | 1 to 3% (low; the lottery) | twin file |
| 11 | The near walk remembers a failed skip | same picture | **new**, weakened by a tried step | 0 to 3% (low) | twin file |
| 12 | `nearest_piece` keeps places, not three whole pieces | same to the bit | **new** detail of the planned stages | 1 to 3% land (low) | Nsight: registers and local writes |
| 13 | Tile jobs polled by one waiter, not each every 200 µs | same picture | **planned** in spirit (one-way handoffs) | evenness, unknown | the slow-frame log, filling 1,1 against 8,2 |
| 14 | Settings' environment reads once, not every frame | same picture | new, tiny | about 0 | not worth a flight |

Leads 1 and 5 overlap a little (both cut `shade_staged` and the faces' pass); 1 and 4 overlap (4 speeds what 1
does less often). Measured in this order, the later ones will show less.

## 3. Each lead

### 1. Light per face kept between frames

- **Where:** `draw.rs:758` clears `face_keys` every frame. `march.wgsl:2112` (`face_key`) measures a face's block
  from the eye's block, so keys change whenever the eye crosses a block. `face_light_staged` (`march.wgsl:2153`)
  then lights every face found, every frame.
- **Why it costs:** the pass is 19% of the card over land, 9% at sea. But what it works out, the sun reaching
  the face's middle and the sky it sees, is read from the face's place, its facing, the sun's direction and the
  store (`march.wgsl:2166` to `2175`). None of that is the eye. Flying forward, nearly every face in view was lit
  last frame with the same answer. The sky's part (five of the six rays) doesn't depend on the sun either.
- **The change:** keys from the block's own place (each axis's low 8 bits; faces are within 127 of the eye, so
  no two in view collide). The table isn't cleared. Each value keeps the frame it was last seen and last lit.
  `listed_staged` lists a face for lighting only when it's new, or its turn has come (a sliver each frame, say
  one in 16, so every face is lit again within 16 frames: tiles landing, edits and a moving sun show within a
  tenth of a second). A small sweep empties faces not seen for a second or so, a slice of the table a frame.
- **Kind:** the same picture to the pixel once the scene is still (the tests' case, with the refresh forced).
  While tiles land or the sun moves, a face's light is up to 16 frames late. David would see nothing, I think;
  it's his call.
- **Risk:** the old "sky light kept" map (dyno B2) lost: its lookups and atomics cost as much as the cones.
  That was in the one-pass march, a lookup a pixel on top of everything. Here the table and its lookups are
  already paid for every frame; only the clearing and the lighting change. Turning fast still brings many new
  faces at once, so the worst frames while turning stay as they are (no worse). Faces lit from coarse cells while
  the near tiles were still being made stay wrong until their turn: hence the sliver.
- **To price:** a twin file where `face_light_staged` leaves at once for seven faces in eight (by the slot and
  a number that changes each frame, such as `camera.glow.z`). The values buffer isn't cleared today, so a face
  mostly keeps an earlier frame's light. The picture is wrong in places (it's a probe for the cost only), and
  the cost is about the lead's. One flight.
- **Guess:** the pass falls to a fifth or less in steady flight: 8 to 14% of the card over land. Medium.
- **If it isn't built:** sky cones from a block's face could read one cell a step (the light is one value a face
  already, so the blend's smoothness buys nothing there). A trade; perhaps 3 to 5% over land, low.

### 2. Ground columns shared by the live levels

- **Where:** `store.rs:112` (`fill_tile`) calls `cells_by` (`on_card.rs:359`) with the tile's own region, and
  `on_card.rs:364` runs `sight_grounds_pass` over that region plus three columns all round. Every tile of every
  level does this for itself.
- **Why it costs:** the five levels' windows sit one inside the other, so a column near the eye is worked out
  five times, and the margin is heavy on small tiles (a tile of blocks: 22 × 22 columns made for 16 × 16 kept).
  Per block flown, by my sum, the levels make 30, 45, 77, 140 and 268 columns for each tile across the window:
  560, of which the 16s' own 268 would cover everything. `sight_grounds_pass` is 15% of the card over land at 255
  registers, spilling.
- **The change:** the 16s' tile already reads its grounds back to the processor (`on_card.rs:417`). Keep them
  (two 16-bit numbers a column) for the ground inside the 8s' reach, and hand a finer tile its grounds by an
  upload in place of the pass. A miss (the fine tile asked for before its 16s' tile) runs the pass as now. The
  grounds are the card's own bits, so the cells made from them are the same to the bit.
- **Kind:** the same picture. One-way: an upload; the read-back is today's.
- **Risk:** memory on the processor's side, about 120 MB at David's window (about 30 MB if only the levels of 1, 2
  and 4 share, for about half the gain). More logic in the tile lane. The alternative, all on the card (a grounds
  window the finer tiles' passes read), is more work for the same saving.
- **To price:** no flight. `LiveStore` already sums each level's columns time (`store.rs:258`, `took`); the
  share of it at levels 0 to 3 in any flight's log is the saving. One flight after building.
- **Guess:** about half of the pass's work while flying: 5 to 8% of the card over land, nothing standing still.
  Medium. It stacks with the planned stage cut (which makes each column cheaper); this makes fewer.

### 3. A narrower hand-over between levels

- **Where:** `march.wgsl:310` (`BLEND_FROM` 0.6) and `1263` to `1269`: past 0.6 of each level's stretch the
  blend is read at both levels and mixed.
- **Why it costs:** the stretch is counted in doublings, so 0.6 to 1 is distances from 1.52 to 2 of every
  doubling: 48% of all distances take two blends. The probe without it measured −9.7 / −12.5 / −10.4%.
- **The change:** `BLEND_FROM` at 0.8 (26% of distances) or 0.9 (13%), as a Render-page slider.
- **Kind:** a trade. The level change happens over a shorter distance; David would look for rings over flat
  water and seams on long slopes (his build 69).
- **Risk:** only the look. No code added.
- **To price:** a twin file with the constant changed. One flight; then his eye.
- **Guess:** pro rata, 0.8 wins a little under half the probe's 10%: 4 to 5%. Medium on speed.
- A further trade, if he likes neither: one level a step, chosen by the pixel's scatter and the step's number
  with the mix's odds. The whole 10%, for grain in the hand-over stretch. Low.

### 4. Shadow and sky rays remember a failed skip

- **Where:** `cone`, `march.wgsl:1449` to `1467`: every step tries a skip over each coarse column from the top
  level down to two over its own, two at a time (up to eight `entry_at` reads), then reads the region's entry
  (`1473`), then blends.
- **Why it costs:** under the land's coarse tops (a ray starting in a valley or on a slope) every try fails,
  and fails again next step for the same column and the same reason. The one-cell probe says what such lookups
  cost: the sun's and sky's rays unpack only two numbers a cell, yet one cell for eight saved them 16% over
  land, so their cost is the entries and band tests, and the skip loop does up to nine more of those a step.
- **The change:** these rays only climb. A try that fails because the ray is under a column's top stays failed
  until the ray leaves that column or climbs over the top. Keep one distance (the soonest of those, a little
  early) and skip the loop until then; forget it when the level read changes.
- **Kind:** the same picture: only tries that would fail are left out, and trying early is harmless. The "past
  the map" return inside the loop is reached at the next real try; the light is the same (nothing past the map).
- **Risk:** the lottery (the sun's cone as the sky's sixth lost 2.6% for no reason in the code). One more
  register in `shade_staged` and the faces' pass.
- **To price:** a twin file; the seven-view picture test. One flight.
- **Guess:** 3 to 8% over land. Low: no measurement splits a cone step into its skip tries and its blend.

### 5. A fine pixel takes its parents' shadow where they agree

- **Where:** `shade_staged`, `march.wgsl:2294` to `2301`: each shown pixel off the faces walks its own ray to
  the sun. Sky light is already handed down from the layer above (`2281` to `2288`); the sun isn't.
- **Why it costs:** `shade_staged` is 29% of the card over land. Most far ground is wholly lit or wholly in
  shadow; only shadow edges need a ray a pixel.
- **The change:** the worth word has 5 bits spare (8 worth, 19 sky). Two of them carry the parent's sun: all,
  none, or between. A fine pixel whose parent and its eight neighbours all say "all", or all "none", takes that;
  any other walks its ray. Parents under a finer layer work theirs out for handing down, as they do the sky's.
- **Kind:** a trade. A shadow thinner than a coarse pixel that falls between the parents' rays is missed, and
  could flicker as the eye moves. The three-by-three test is there to make that rare.
- **Risk:** David's eye on thin shadows. Parents now walk shadow rays they skipped when hidden.
- **To price:** a twin file (the bindings don't change). One flight; then his look at a shadow's edge on a far
  slope. Section 5 has a count that prices it without building.
- **Guess:** 4 to 9% over land. Low: I don't know how much of `shade_staged` is far shadows now.

### 6. `live_slot` without whole-number remainders

- **Where:** `march.wgsl:215` to `219`: `((cy % side) + side) % side`, twice, with `side` read from the header.
  The side is 672 or 336 (`store.rs:236`), not a power of two.
- **Why it costs:** four whole-number divisions by a number the compiler can't know, for each of a blend's four
  columns and each skip try on a live level. Cards have no instruction for it; each is a long chain, with naga's
  guards round it, in front of the read it addresses. That is the shape of the top stall, a group waiting on its
  own last sum.
- **The change:** the live index wraps at the next power of two (1,024 or 512 a side; 80 MB against 36) and the
  shader masks. `card_store.rs:397` wraps the same way.
- **Kind:** the same picture.
- **To price:** a twin file first, the store untouched: the remainder by a float floor and one fix-up (exact
  for these sizes). If it wins, the power of two is the real form. One flight.
- **Guess:** 1 to 4%. Low. With it, the header's six words could ride in the camera's uniform; smaller still.

### 7. The far loop's room

- **Where:** `march`, `march.wgsl:1151` (`before`), `1318` (`faces_light(before, d)`), `1165` to `1170` (the
  floor's eight registers), `1263` to `1269` (the second blend while the first is held).
- **What's alive, by my count** (not measured): about 60 across the loop (the ray, three colours gathered, the
  walk's state, `before` at 12, the floor's 10), and at the peak `look` (12) held while the hand-over's blend
  runs (its sums 12, a cell unpacked 12, weights and places about 10). That is near the 125 measured, and **it
  is the far march's own**: the near walk's state is dead by then.
- **Changes, each the same picture:**
  - `before` keeps its light, not its six face shares: `faces_light(look, d)` worked out when the sample is
    taken, one step sooner, the same sum on the same numbers. Five registers off the whole loop, to the bit.
  - The six shares as three sums inside the blend (the lit part, the seen part, the all-round part: each is
    a straight sum over the shares, so it blends and mixes the same). Nine or so off the peak. Rounding may
    differ in a last place; the picture test would say.
  - The floor's two colours as half floats in three words, or worked out again after the loop from the two
    places kept. Five to eight more.
- **What it would take:** these give about 105 (19 groups for 16). Under 80 needs the blend's own sums gone,
  which is the texture units. Under 64 I can't see without them.
- **To price:** a twin file each; the registers from Nsight's window. **Guess:** 1 to 3%. Low: the lottery.

### 8. Read-only bindings, for a new reason

- **Where:** `march.wgsl:73`: `starts` is declared read-write (the start pass writes it) and every march and
  shade pass binds it through `name_the_bindings`.
- **Why it may cost:** wgpu 30 on DX12 puts a UAV barrier between two dispatches for every buffer bound
  read-write in both (wgpu-hal `dx12/command.rs`, the `STORAGE_READ_WRITE` to itself case). So every pair of
  dispatches in the frame has one, whatever else they share. Whether this driver drains the card for a barrier
  on one buffer I can't tell from here (section 5).
- **The change:** the start pass writes through a binding of its own; the rest read `starts` read-only. The
  same for anything else only read after it's made.
- **Status:** "read-only bindings where the march only reads" is in dyno section 2, unticked, for the driver's
  read paths. The barriers are another reason to do it before more passes are added.

### 9. Lists by what a pixel needs

- **Where:** `listed_staged` (`march.wgsl:1997`) lists every marched pixel; `shade_staged` (`2248`) runs them
  64 at a time in list order.
- **Why it may cost:** a sky pixel does nothing in the shading but is listed, read (48 bytes) and launched.
  A pixel on a lit face does two lookups; its neighbour in the same group may walk a shadow ray and five cones,
  and the face's lane idles until it ends. Pixels in flight are 10% of what the card holds.
- **The change:** `march_hit` writes a sky pixel's picture itself and marks it not for listing. The listing
  fills two lists, cone pixels from the front and faces' pixels from the back, each launched by its own count.
- **Status:** tiles by type and warps kept alike are planned (dyno B7, B10). Builder found the shading's cost was
  repeated work, not idle lanes, before light per face; it may read otherwise now. Count first (section 5).

### 10. The listing as the march's tail

- `listed_staged` re-reads every pixel's record over the whole layer (3% of the card) and is a pass to drain.
  `march_hit` knows at its end whether its pixel is live; after a `workgroupBarrier` it can take its place in the
  list and find its face. Three passes fewer a frame. The same picture.
- **Risk:** the lottery, squarely: code added to the biggest shader. Its tail needs few registers, but the
  compiler decides. A twin file prices it. **Guess:** 1 to 3%, low.

### 11. The near walk remembers a failed skip

- `hard`, `march.wgsl:840`: every block step tries the columns of 64, 32, 16 and 8 first. Low over the ground
  all four fail, step after step. The same memory as lead 4 (valid until the ray leaves the column of 8, or
  climbs over the lowest top that failed) leaves those reads out. The same picture; trying early is harmless.
- **Status:** Builder tried the opposite (more levels to skip by) and found the near blocks don't cost in the
  stepping (dyno B1); the water jump lost to its own lookups. This removes lookups, so it isn't ruled out, but
  her finding caps it. **Guess:** 0 to 3%, low. A twin file.

### 12. `sight_grounds_pass`: where the room goes, and where to cut

- **Where the registers go, by reading:** `nearest_piece` (`recipe.wgsl:849`) keeps three whole `Piece`s alive
  across its loop (`best`, `wet`, `stretch`: 17 words each, a 12-word river point inside), with a fourth and two
  more points (24) made each turn: over 100 on its own. `compressed_bank` (`1009`) holds two points and a dozen
  running values. `maps_at` (`337`) hands nine blended fields on as `Here`, alive until the uncarved height.
- **A first move, the same to the bit:** the three pieces kept as places (which reach, which point, how far
  along, the distance and side: five words), and `blend_point` run once after the loop for the one that won.
- **The cuts, each handing only forward:** (a) the maps: `maps_at`, 10 words a column; (b) the shape: `parts`
  to `uncarved_from` (`685`), one height; (c) the river: `nearest_piece`, the winner's five words; (d) the carve:
  the rest of `carved_from` (`1252`) with banks and lakes. `carved_from` already takes the uncarved height as
  its only input from before, so (b) to (c) is a clean line today.
- **Status:** the stage cut is decided (2026-11-10), third in the order. Lead 2 is separate and can go first.

### 13 and 14. The processor's side

- **13.** Each tile is five or more submissions (`on_card.rs:373` to `436`: grounds in pieces, a read-back,
  choices, cells in pieces, a read-back), each waited for by its own job calling `device.poll` every 200 µs
  (`on_card.rs:85` to `95`). With ten tiles in the making that is up to 50,000 polls a second on the loading
  threads, each taking wgpu's locks beside the frame's own submit. One waiter for all would do. Whether it shows
  in the frame's evenness I can't tell from reading (section 5).
- **14.** `draw.rs:642` to `655`, `runner/mod.rs:558` to `562` and most `sight_*` getters in `look.rs` read
  environment variables every frame. Read once at the start. Microseconds; tidy rather than fast.

## 4. The plan's critique

**The order: split, then texture units, then the ground's stages.**

- **The split (near walk and far march apart).** By my count (lead 7) the 125 is the far loop's own peak: the
  last sample, this one, and a second blend in the hand-over. If so, the far pass stays near 125 and only the
  near pass gets lean. The split would then pay through other things: the far march run only over pixels that
  go far, packed (another list), and near and far rays no longer sharing a group. Those are real, but each new
  pass is a drain, and every pass added so far has cost the typical frame (+13%, +9%, +4%, +6.7%) while helping
  the worst. The plan sizes it first with two probes and their registers. That is the right check, and I'd hold
  the build until the far-march-alone probe reads well under 125. If it reads 110 or more, texture units first.
- **What the split doesn't count:** the near walk's hand-over is a dozen numbers (what it gathered through
  water, how much is left, where it stopped), so the record grows or a second one appears; pixels whose near walk
  met a face (most, in the heavy views) never reach the far pass, so its list is short there and full on the
  typical frame, where the split's extra passes cost most.
- **Texture units through bricks.** The right big bet: the far march's own blend is 28% by the one-cell probe
  and the registers follow it. What the plan doesn't count:
  - **Not the same picture to the pixel.** Hardware filters blend with weights of 8 bits. Fine to the eye, but
    it's a stated-tolerance step, not a pixel-equal one. Worth saying before the tests fail.
  - **It isn't plain trilinear.** Colour and faces are weighted by surface and divided after, so the bricks hold
    them already weighted: twelve numbers, three fetches a blend, and surface runs to 4, so 8-bit textures band at
    thin coverage. Half floats (filterable everywhere) make a cell 24 bytes for 16; 16-bit whole-number textures
    need an optional feature.
  - **Borders have to be kept true.** "Below the band is solid, above is empty, not made yet means read the
    coarser level" all live in branches today. In bricks they live in border cells, written again whenever a
    neighbour tile lands. More uploads while flying, and more work on the tile lane.
  - **The ceiling is generous.** One cell for eight also ends rays sooner and drops three of four entry reads.
    Bricks keep a table read and three filtered fetches. I'd expect half the ceiling, not all.
  - **Price it with a stand-in first:** the one-cell twin with three `textureSampleLevel` reads of the block
    textures added at each blend (between two shrunk versions, so each is an eight-texel filtered fetch). No new
    bindings; one flight. That reads closer to bricks' real cost than the bare ceiling.
- **The ground's stages third.** Fine as an order for the registers. But lead 2 halves the pass's work without
  touching its shape, and lead 12's first move is small. I'd do those two before the split, not after the
  texture units: 15% of the card over land is the largest cost with a same-picture answer already in sight.

**What's missing from the plan:** light per face kept between frames (lead 1; the plan's "light computed once"
items are all per cell or per map, none uses the face table that now exists); shadows handed down as sky light
is (lead 5); the skip tries in the shadow and sky rays (lead 4; the plan says "their cost is their own" and
leaves it); the hand-over's width as a setting (lead 3); the barriers (lead 8).

**Does a measured fact contradict a step?** Two things rub:
- The plan reads the ~10% in flight as "the drains and tails of twelve passes", and its next step adds passes.
- The lean blend for the sun's and sky's rays was noise, yet one cell for eight saved them 16% over land. So
  their cost isn't unpacking; it's the lookups (entries, band tests, remainders). Bricks for "the rays' two
  numbers alone" were set aside as a quarter of the win, but leads 4 and 6 go at the same cost without bricks.

## 5. What couldn't be judged without measuring

- **How full the groups run.** Each pixel's steps are already in its record (`hits`, the third word's high
  half). A small pass, or the processor on a read-back, can sum and take the most over each 8 × 8 group: the sum
  over 64 times the most is how full the group ran. One number a pass, by leg. It prices leads 9 and 10, the
  split's packing and the ray queue at once. (It's the "slowest-ray view" of dyno section 1, unticked.)
- **New faces a frame** (lead 1): the faces' count is on the card; keep keys from the block's own place for one
  flight and count the faces not in last frame's table. Flying, and turning.
- **Parents that agree** (lead 5): in `shade_staged`, count fine pixels whose nine parents' sun values are all
  0 or all 1. That share of the fine layer's shadow rays is the saving.
- **Near against far inside `march_hit`:** Builder's two planned probes, with registers. Decides the split.
- **Whether a barrier on one buffer drains the card:** a flight with `starts` read-only and the passes' gaps
  summed (the per-pass timer against sight's whole) before and after.
- **Registers after each of lead 7's changes:** only Nsight's window shows them.
- **The tile lane's weight on the frame:** a flight with `INCUS_SIGHT_FILLING=1,1` against `8,2`, reading the
  slow-frame log's "handing the card its work" and "sight" times and the 99th percentile.
- **What `sight_grounds_pass` spills:** Nsight's local-memory writes before and after lead 12's first move.

Not looked at closely: `models.wgsl`, the objects' draw, the bake's own passes, the lights' flood.

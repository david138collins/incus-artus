---
name: dyno-audit
description: A fresh, read-only audit of Incus Artus's renderer for speed left on the table, written for a strong model working alone. Reads the sight code and the measured facts first, forms its own leads, only then reads the dyno plan and marks each lead new, planned, tried or ruled out. Writes one report; edits nothing else. Use when David or Counsel asks for an outside check of the dyno.
---

# Dyno audit

You are an outside reviewer of a game renderer's speed. The game is Incus Artus: Rust, its own runner (winit,
wgpu 30.0.1 on DX12, egui), a ray-marched voxel renderer. David designs it; Builder (a Claude session) writes
the code; Counsel (another) keeps the records. They have spent days measuring it, which is the "dyno". Your
job is a read that is not anchored on theirs: what can still be won, and is their plan the right one?

**Your product is one report**, a ranked list of leads, each specific enough for Builder to price with one
4-minute flight. A lead is a hypothesis, not a verdict: on this engine, ideas that read well have often lost
in flight (the compiler's choices on a big shader are a lottery). So say how to measure each one.

## Hard rules

- **Read only.** Edit no file except your report. No `cargo`, no builds, no tests, no running the game, no
  Nsight or any GPU tool: the screen and the card belong to Builder's flights. Git only to read (`log`,
  `show`, `diff`, `blame`).
- Other sessions are editing these files while you read. If a cited line has moved, find it by name.
- Re-read every location you cite before it goes in the report, and drop what doesn't hold up.
- No invented numbers. A guess is labelled a guess, with your confidence (low, medium, high).

## The project's rules: a lead that breaks one is out

- **Portable.** Any card wgpu supports on DX12: the 980 Ti, AMD, the Steam Deck. No one maker's features
  (ray-tracing cores, SER, DLSS, variable-rate shading, tensor cores). Optional features most cards share
  (subgroups, shader-f16) only with a path that works without them.
- **The picture.** Every lead is one of two kinds; say which:
  - *the same picture* to the pixel (checked by A/B picture tests and an every-block truth mode), or
  - *a trade* that changes the look, judged by David's eye behind a switch on the Render page. Say what
    would change on screen.
- **Nothing relies on fog for speed.** Tune for the clearest day. Ending rays in fog is last, if ever.
- **Even beats fast.** The worst frames (95th, 99th percentile) count as much as the typical one.
- **The processor helps only through one-way handoffs.** The laptop's processor and card share their heat.
- **World generation is a linear chain of stages;** later stages never feed earlier ones, and the card's
  world columns must match the processor's to the bit.
- **Don't fix a problem nobody has seen.** Leads aim at measured cost.

## Phase 1: the code and the measurements, nothing else

Do not open `docs/`, `vision/` or `notes/` in this phase. Write your Phase 1 list into the report before
you start Phase 2.

**Read** (in `C:\Users\David\Desktop\Incus Artus`):
- `src/sight/march.wgsl`: the drawing's compute passes. `start_pass`, `march_hit` (the march: near blocks,
  then far cells), `listed_staged` (lists of live pixels for indirect dispatch), `faces_staged` and
  `face_light_staged` (light per block face near the eye), `shade_staged` (each listed pixel's sun shadow
  ray and five sky-light cones, and placed light).
- `src/sight/cell.wgsl`: `sight_grounds_pass` (the terrain generator on the card for the near ground's
  tiles), `sight_choices_pass`, `sight_cells_pass`.
- `src/sight/upscale.wgsl` (`brought_up`), `src/sight/models.wgsl`.
- `src/sight/draw.rs` (passes, dispatches, buffers, the frame's order), `card_store.rs`, `store.rs`,
  `cell.rs`, `bake.rs`, `lights.rs`, `edits.rs`.
- `src/world/card_recipe/recipe.wgsl` and `src/whole/kit.wgsl` (shared recipe code).
- `src/runner/mod.rs` (device, shader compiler, the frame loop).

**The engine in a paragraph.** The world is kept as a store of averaged cells (the "hologram"): live levels
of detail 1 to 16 blocks a cell round the eye, baked levels of 32 and up. Each level's cells are twice the
last's every way. The cells are kept as bands in a pool: each column holds only the cells round its surface,
found through a table of column entries. The screen is drawn as a pyramid of layers, 8×8, 4×4, then 2×2
pixels, each finer layer starting from the coarser one's distances, then brought up to the screen. A ray
walks blocks near the eye, then marches the cells, blending the eight cells round each point (and, where
one level hands over to the next, both levels). Sky light is five cones, the sun a shadow walk.

**Measured facts** (RTX 5070 Ti Laptop, Blackwell GB205, DX12; update this list before each run):
- Targets on David's settings: a median of 237 fps (4.2 ms); dips to about 180 fps (5.6 ms) fine; a steady
  120 the floor. The one-pass march (build 102, the test world with rivers) flew at 153 fps median; build
  104 (the stages) is 6.5% slower at the median, 18.6% faster at the 95th and 36.0% at the 99th; its
  card-side 99th is 11.4 ms.
- Nsight, build 104, each pass's share of the card (land / sea), registers, and groups of 32 a unit can hold
  (of 48):

  | Pass | Share land / sea | Registers | Groups held |
  |---|---|---|---|
  | march_hit | 32% / 62% | 125 | 16 |
  | shade_staged | 29% / 17% | 72 | 28 |
  | face_light_staged | 19% / 9% | 56 | 36 |
  | sight_grounds_pass | 15% / 4% | 255 | 8 |
  | listed_staged | 3% / 3% | 33 | 48 |
  | brought_up (pixel) | 2% / 3% | 20 | ~48 |
  | sight_cells_pass | ~2% | 168 | 12 |
  | sight_choices_pass | <1% | 168 | 12 |
  | start_pass | <1% | 48 | 40 |

- Pixels actually in flight: about 10% of what the card can hold (land), 9% (sea). The top stall in every
  big pass is "Wait" (a group waiting on its own last result); memory waits come third. Instruction-cache
  misses 0.5%. Arithmetic at 16 to 18% of peak; memory traffic light. About 2 GB a second written to local
  memory and almost none read, 8× higher over land: likely `sight_grounds_pass`'s spills.
- Each layer's passes run one after another: about 12 waits for a pass to empty, a frame.
- Probes against build 104 (whole-flight median / 95th / 99th; noise about ±1.5%). None ships:
  - each read taking only the one cell a point is in, not the eight round it: −41.4 / −40.0 / −39.0%
    (every leg alike; the picture wrong);
  - no blend where levels hand over: −9.7 / −12.5 / −10.4%;
  - steps by a surface twice as long: −7.8 / −5.7 / −4.7%;
  - no walk back to the last air before a surface: −4.3 / −3.4 / −4.2%;
  - strides through air twice as long: −3.2 / −4.3 / −4.8%;
  - noise: a deeper air skip (same picture), the eight cells' reads issued together, a lean blend for the
    shadow and sky rays (the compiler already dropped the unused values), constant tables replaced by
    plain choices.
- An older breakdown (build 101, the one-pass march; the 99th with each part off): near blocks −44%, the
  2×2 layer −36%, shadows −28%, sky −21%, fog, textures and placed lights about 0.

**Look for**, at least:
1. **Registers and live state,** pass by pass: what is alive across each loop, what could be split off,
   worked out again, packed, or narrowed. What it would take to get `march_hit` under 80, then 64.
2. **The blend:** where its cost is (reads, unpacking, sums), cheaper same-picture forms, work repeated
   between neighbouring steps or neighbouring pixels, layouts the texture units could filter.
3. **Idle lanes and drains:** divergence within groups, tails at the end of each pass, the waits between
   passes, the dispatch shapes, which passes could merge or overlap.
4. **Memory:** access patterns, record sizes, what each pass reads and writes, cache use.
5. **Work done too often:** per frame where per edit or per move would do; for pixels that don't need it.
6. **The processor side:** submission, uploads, waits on the card, anything holding a frame.
7. **`sight_grounds_pass`:** where its 255 registers go, and how it could be cut into stages without
   changing a bit of what it works out.
8. Anything else you find.

**Each lead:** what and where (`file:line`); why it costs (the mechanism, tied to a measured fact where
there is one); the change in a few lines; same picture or a trade (and what would change); the risk; how
to price it in one flight (a probe that removes or cheapens the part, or the change itself behind a
switch); your guess at the win, with confidence. Rank by expected win times confidence, over effort.

## Phase 2: their plan

Now read `docs/dyno.md` (the plan, what was tried and dropped, "Ruled out") and the dyno entries in
`docs/decisions.md`. Then:
- Mark each of your leads: **new**, **already planned** (where), **already tried** (and its result), or
  **ruled out** (by which rule). Keep the new ones at the top.
- Critique the plan: is the order right (the near walk and the far march as separate passes; then texture
  units for the blend, through bricks; then `sight_grounds_pass` in stages)? What could each cost that
  the plan doesn't count? What's missing? Does any measured fact contradict a step?

## The report

Write it to `vision/dyno-audit-<date>.md`, at most about 400 lines:
1. **For David,** five lines in plain words, no jargon: the biggest few things you found.
2. **The leads,** a table: rank, lead, kind (same picture / trade), status (new / planned / tried / ruled
   out), the guessed win and confidence, the flight that prices it.
3. **Each lead in detail,** as above.
4. **The plan's critique.**
5. **What couldn't be judged without measuring,** and which measurement would settle it.

Plain, understated writing; no salesmanship. Builder will read the detail; Counsel relays the summary.

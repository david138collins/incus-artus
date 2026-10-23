# Incus Artus — Decisions Log

This file records every design and technical decision we've agreed on, and why.
When something changes, we update it here so there is always one place that
says "this is how the game works right now."

Status tags: **[DECIDED]** agreed · **[OPEN]** still needs an answer · **[LATER]** noted, not built yet

---

## Units & Coordinates

- **[DECIDED]** 1 block = Minecraft scale (1 m).
- **[DECIDED]** Creature heights: dwarf 1.5, human 1.7, elf 2.1 (elves crouch through human/dwarf doorways).
- **[DECIDED]** +X = north, −X = south, +Y = east, −Y = west, +Z = up. Origin (0,0) is the centre of the world circle.
  - Note for code: this is a *left-handed* system. Bevy (the engine) is right-handed and Y-up,
    so we convert only at the very edge where things get drawn. All game logic uses our axes.
- **[DECIDED]** Vertical range Z = 0 (bedrock) to Z = 399. 400 layers.
- **[DECIDED]** World is a circle ~72,000 blocks across (radius ~36,000), ~70% water (tunable later).
  Float precision is fine at this size.

## Chunks & Storage

- **[DECIDED]** A chunk is a 32 × 32 × 400 column. Internally split into stacked 32³ *sections*
  (13 sections; the top one is partly unused). Chunks are only a storage detail —
  no terrain or biome logic is based on them.
- **[DECIDED]** Block type IDs are 16-bit. Bulky per-block state lives in a separate,
  optional structure that only exists for blocks that need it.
- **[LATER]** Performance techniques planned: see "Performance plan" below.

## Quartile Placement System

Reference: `Art/quartile minecraft illustration.png`

- **[DECIDED, LATER to build]** Every block cell is split in half along each axis → 8 *octants* (½ × ½ × ½).
- **[DECIDED]** A *quarter block* (cut stone, board, brick, building stone) is a 1 × ½ × ½ bar = 2 octants
  = ¼ of the block's volume. It can lie along any of the 3 axes, in any of 4 positions →
  12 possible bar slots per cell. Bars may overlap (share an octant) to make corners.
  Quarter blocks replace stairs, slabs and roof pieces. Dirt slabs exist.
  - Example: stairs = 3 bars stacked (¾ of the cell filled).
- **[DECIDED]** Item placement: every exposed surface — top, side, any step of a stair — offers
  its ½ × ½ quarters plus a centre spot. Small items (mug, flask, bowl) take one quarter;
  medium/large items take the whole face. Centre and quarters are mutually exclusive.
  A 2×1 table has 10 spots (4 quarters + 1 centre per block). Picture frames hang on wall quarters.
- **[DECIDED]** Item sizes:
  - Small items need a quarter face, and can also go in the centre.
  - Large items need a full flat block face. They can span two blocks if both faces are suitable.
  - Centre spots only exist on flat full faces, so a stair cell has no centre.
- **[DECIDED]** A cell can mix materials. When bars overlap, the bar placed first keeps the shared octant
  and the new bar loses that half. Mining a mixed cell destroys it all (blocks generally drop less than they're worth).
- **[DECIDED]** You can't put anything on a surface that already has an object on it.
  Objects placed against blocks protect those blocks from mining until the object is moved.
  Example: a 2-block spear leaned on an octant face against a 2-tall wall.
- **[DECIDED]** Pallets show stored blocks/stone visually, in miniature and compressed.
- **[LATER — REMIND DAVID]** Should quarter blocks ever spawn naturally in world gen? Decide while experimenting with world gen.
- **[LATER]** Floating cone marker showing which face/quarter you're aiming at.

## Blocks (see also `docs/block-properties.md`)

- **[DECIDED]** A block = **material × inclusion**. Material (granite, rich loam…) gives hardness, sound, colours.
  Inclusions are an optional overlay layer (ores first; the same mechanism can carry other overlays later).
  Blocks also carry a tag allowing true texture *variants* later if a good case appears.
- **[OPEN]** Name for the "what kind of thing" dimension (natural granite vs granite cut stone vs granite brick) — see chat.
- **[DECIDED]** Colour by **palette swap**: textures are drawn in shade *slots*, and each material
  fills the slots from the strict palette. A dedicated colour session with David comes before the real textures.
- **[DECIDED]** Inclusions list several ores with percentage chances (galena → lead, sometimes silver).
  Density and grade are fixed at "planet forming" world gen.
- **[LATER]** Staged mining: rock (small, "pennies") → boulder ("nickels") → boulder with a pick,
  or a full block with hammer and chisel. Speed depends on tool, material, and the miner's RPG skill (NPCs and player alike).
- **[DECIDED]** Random face variants (e.g. some logs with knots) are feasible. The variant is picked from
  the block's position, so it needs no storage.
- **[DECIDED]** "Constructed" is a yes/no bit, plus optional richer ownership (pillagers, quest tags) for the few blocks that need it.
- **[LATER]** Chemistry: blocks and regions carry chemical properties (e.g. soil). Irregular weather
  (milder/harsher winters) moderates plant growth through them.
- **[DECIDED]** Fewer than 65,000 block types — a few thousand is plenty.

## Regions (buildings, fields, pastures, rooms, "cells")

- **[DECIDED]** Areas are stored as shapes, not as a flag on every block. Selecting a room, area or cell
  (an in-mountain room) marks it as warehouse, pasture, meeting area, etc.
- **[LATER]** In isometric view, mousing over shows the area's border; it can be resized; assigned workers act on it when in season.
- **[DECIDED]** A solo player can farm without ever defining an area.

## Isometric View

- **[LATER]** The camera is far away, so distant-style simplification applies (no face detail needed),
  but people must stay easy to locate.
- **[LATER]** Height-slice scrolling to see inside buildings (needs cut caps, see the rendering notes).
- **[LATER]** See-through indicators on people (at least when their character sheet is open), visible through walls and trees.

## Multiplayer
- **[LATER, as late as possible]** Alpha-tester link: first an in-game note box that posts to a Discord webhook
  (with screenshot, log, position, settings); later two-way chat; later a dev "drop-in" by joining the tester's game as
  a guest. That last one is the same feature as hosting a friend: LAN-party / D&D-group scale, host-a-friend,
  no server infrastructure. A built-in webhook address can be abused; hide it behind a tiny relay when it matters.
- **[DECIDED]** Keep the refactor easy: every player action is a request (already true for break/place);
  players get IDs and their own input instead of one global player (to do in step 6).

- **[DECIDED]** Not a goal. A basic hookup may come later for testing. For now, player actions go through the
  event bus as requests ("player wants to break X"), so a network layer could inject them later without rewrites.

## Time

- **[DECIDED]** One TimeControl resource. Simulation is fixed-tick; rendering/UI variable.
- **[DECIDED]** Max speed 4× (dev tool; may not ship to players). Pause must be possible
  (dev + manager screen + optional setting).
- **[DECIDED]** Calendar target: year = 4 seasons × 7 days; day = 15 min light + 15 min dark.
- **[DECIDED]** Pawn movement runs on the fixed tick, input polled every frame,
  position interpolated between ticks for display.
- **[DECIDED]** 30 simulation ticks per second (tunable on the debug panel). Heavy systems
  can run every Nth tick.

## Look & UI

- **[DECIDED]** 640×480 is the *UI art* resolution only (maybe 1280×960 later). UI is integer-upscaled.
  The 3D world renders at native resolution.
- **[DECIDED]** Limited palette; full colour reserved for special effects.
- **[OPEN]** The master palette isn't chosen yet (`lorecraft/art/Palettes/masterpal.gpl` is an experiment, not final).
  Current textures are placeholders; colours get coordinated once a visual style is visible. Don't check art against it.
- **[DECIDED]** Block faces are 32×32 pixels.
- **[DECIDED]** Placeholder UI during development: black boxes, magenta outlines, bright pink text.
- **[DECIDED]** Plain lighting for now. Pretty lighting later.
- **[LATER]** Widescreen presentation decided once the shell runs.
- **[LATER]** Bottom bar: equip section · three fluid spheres · quickbar.
  Sphere order, left to right: **blue energy** (next to equip, because equip can hold spells),
  **red blood**, **green stamina**. Can change later.
- **[DECIDED]** Game name is **Incus Artus**.
- Reference art: `lorecraft/art/references/inventoryequipsave.png` (personal/party view, newest),
  `lorecraft/art/bad art/three man.png` (manager view), `lorecraft/art/personaltest1.png` (earlier in-world overlay).

## Textures & Assets

- **[DECIDED]** `Art/` is David's messy workshop. `assets/` is the delivery folder: finished files go there
  and the game reads only from there. Block textures: `assets/textures/blocks/<name>.png`.
- **[DECIDED]** Block textures are exactly 32×32 PNGs (transparency allowed). Wrong size or missing →
  magenta/black checkerboard, so problems are obvious.
- **[DECIDED]** Faces: `all`, or `top`/`side`/`bottom`, or six faces. Side faces: image top = up.
  Top/bottom faces: image top = north.
- **[DECIDED]** Textures must tile seamlessly (greedy meshing repeats them across big surfaces).
- **[DECIDED]** Each face of a block has a texture rotation: `fixed` (default) or `random` (each block
  turned 0/90/180/270°, picked from its position so it never changes). Grass top is `random`. Use it only
  on textures with no "up" direction. Randomly rotated textures must tile seamlessly in all four turns.
- **[DECIDED]** Claude points out factual, functional problems in delivered art (seams, sizes, readability).
- **[DECIDED]** Each soil type gets its own texture; the grass edge is one shared transparent overlay drawn on top of any soil.
- **[LATER]** Distant (shrunk) textures: Claude generates the small versions for a few block types; David processes and touches them up; hand-made versions can then replace the generated ones.
- **[LATER]** Random texture variants, animated textures.
- **[LATER]** Trees generate with quarter logs.
- **[LATER]** A blueprint / structure suite for David to define trees and vanilla structures.

- **[DECIDED]** Optional **Panini projection** (full-screen pass): wide FOV with a natural-looking centre. Tunable
  distance and crop-to-fit, on the Camera page. The FOV setting is vertical.
- **[DECIDED]** See-through blocks, both using texture `grid.png`: drawn on both sides with hard-edged cut-out
  transparency (pixels fully clear or fully solid, no grading), walk-through, targetable (switch per block),
  replaceable (placing a block into one swaps it out). Two kinds:
  - `incus:grid`: every cell visible (faces between grid neighbours are drawn).
  - `incus:outline`: only the outside skin of a volume, so you see the far side's inner face through it.
    For defining exact 3D regions in the creative/structure editor.
- **[DECIDED]** Generated mip levels average colour only over visible pixels, so clear pixels never darken
  coloured ones in the distance.
- **[LATER]** In-game structure editor ("temple"): an adjustable grid to build vanilla structures in, with data-point
  blocks (village-piece connectors for wave-function-collapse generation, tree structures for world gen and growth).
- **[DECIDED]** Random texture rotation is switched per block on the Blocks page: one switch for blocks that look
  the same all round, separate top / sides / bottom switches for blocks with a distinct top.

## Data & Modding

- **[DECIDED]** Text IDs in data and save files (e.g. `incus:granite`); numbers only at runtime.
- **[DECIDED]** One file per block, readable folder layout like old PC games.
- **[DECIDED]** Live reload of data files, built so it can be removed cleanly.
- **[DECIDED]** Modding is a long-term goal — content is laid out so mods can add folders alongside the base game.
- **[DECIDED]** Data files are **TOML**: plain text, comments allowed, X-COM-style readable.
- **[DECIDED]** Models: Blockbench now, a real 3D modeller later. We'll use a loading path that
  works for both (likely glTF, which both can export).

- **[DECIDED]** (Step 7) Blocks are defined in `assets/data/<pack>/blocks/<name>.toml` → ID `<pack>:<name>`; the
  base game's pack is `incus`. Only air is built in. Format and settings: `docs/data-files.md`.
  `look = solid | grid | outline` sets the usual behaviour; `solid`/`targetable`/`replaceable` can override it.
- **[DECIDED]** Misspelt settings are errors (reported with file and line); unknown face words are warnings.
  Mistakes never crash: a bad file keeps the block's last good version; a missing test-world block gets a magenta
  placeholder; a deleted file's block stays until restart (block numbers never change while running).
- **[DECIDED]** Live reload covers data files *and* block texture PNGs (checked every second, real time).
- **[LATER]** A "write this tweak into the block's file" button on the Blocks page (keeping comments).

## Saving

- **[DECIDED]** No save/load-game. Just worlds, Minecraft-style world folders.
- **[DECIDED]** Only changed chunks are saved; untouched terrain regenerates from the seed.
- **[DECIDED]** Settings in readable text; tuning saved as named presets (save / edit / save as).
- **[DECIDED, built 2026-10-22]** World folders in `worlds/`: each part of the game saves its own file (clock, terrain,
  fate, sim, player), chunks store block *names*, every file carries a format version. Save page on the debug panel.
  Details in `docs/simulation.md` section 9.
- **[DECIDED, built]** Three simulation levels (local / rich / thin), hooks, journeys, Fate. See `docs/simulation.md`.

## Event Bus (step 6)

- **[DECIDED]** Two kinds of message: **requests** (someone wants something; the owner checks it and carries it
  out or refuses it *with a reason*) and **announcements** (facts that happened, stamped with their tick).
  Systems never reach into each other to change things.
- **[DECIDED]** Physical world changes happen on ticks, so a paused world is truly frozen; a player's break/place
  while paused is refused ("TimePaused").
- **[LATER]** **Plans** (dig designations, region outlines, blueprint ghosts snapping under the cursor) are a separate
  layer over the world and work instantly while paused (plan while paused, settlers act when time runs). Ghosts
  are drawing only. The structure editor's edits will be marked as coming from the editor and may apply while paused.
- **[DECIDED]** Every player has a `PlayerId` and their own input; the one at this keyboard is the `LocalPlayer`.
  Every request says who sent it.
- **[DECIDED]** Event log: the Events debug page (filters, freeze, clear) and a file per session in `logs/`
  (newest 20 kept), for bug reports.

## Dev Tools

- **[DECIDED]** Debug panel built with egui. Every tunable exposed the moment it exists.
- **[DECIDED]** Event log on the panel, plus whatever diagnostics I need.
- **[DECIDED]** Debug settings can be saved and loaded as named presets (TOML files in `presets/<category>/`).
  Quick version now (Player page); full save-as / editing / per-actor profiles with data loading (step 7).
- **[DECIDED]** An **everything preset** (Overview page) saves and loads every debug setting at once, so a good
  combination can always be recovered exactly (the first change that made a later one work isn't forgotten).
  One file in `presets/everything/`, one section per group. Every new tunable joins a group.
- **[DECIDED]** Defaults: 100° vertical FOV with Panini on (distance 0.3, crop 1.0).
- **[DECIDED]** Shrunk (mip) versions of textures keep the most opaque pixel of each 2×2, so thin cut-out lines
  never vanish in the distance or at steep angles.
- **[DECIDED]** Design philosophy: play with every setting; odd settings that feel neat might "be something".
  Tools should make experimenting and keeping the results easy.

## Motion & Kinematics

- **[DECIDED]** Motion factors are shaped with ADSR envelopes (attack, decay, sustain, release, each with a
  curve), edited on a draggable graph like a synth or Ableton automation.
- **[DECIDED]** Keys: **Shift** = run, **double-tap Shift** = sprint, **Ctrl** = crouch (flying keeps Shift = fast,
  Ctrl = down). Tuned so a clean jump clears a 2-block gap from a walk, 3 from a run, 4 from a sprint.
- **[LATER]** Landing slide (carry momentum on touchdown so careless landings can slide you off an edge). Tried a
  grip-reducing landing envelope; it didn't achieve much. Probably needs real momentum and mass; revisit with the
  kinematics system.
- **[DECIDED, built 2026-10-22, to tune]** **Momentum**: a charge built while running or sprinting (momentum
  envelope attack) and discharged after you stop pushing (release; frozen in the air, so it carries through a jump
  and discharges after landing). While charged, slowing down only gets `momentum_grip` of the normal grip, scaled by
  how fast you're going (a jog carries little, a dead sprint a lot). Pushing against it (backpedalling) brakes
  `backpedal_brake` × harder. The "woah woah woah" stop; meant to make charging out of a sprint a skill.
- **[DECIDED]** Player movement physics and world-gen "geology" stay separate systems (different scales and jobs);
  a shared kinematics layer will grow for bodies that move: players, NPCs, animals, carts, falling trees.
- **[DECIDED]** A `Landed` message (where, and how fast you hit the ground) fires on every touchdown: a hook for
  landing sounds, fall damage, stumbles.
- **[DECIDED]** Anything that changes on ticks and moves the camera is drawn blended between ticks (crouch included).
- **[DECIDED]** Crouch: shorter body (1.2), lower eyes (1.05), slower (× 0.3), blended by a crouch envelope;
  can't stand up without headroom; won't step off edges while crouched on the ground (switchable).
- **[LATER]** Windows Sticky Keys pops up after 5 quick Shift presses; the game should switch that shortcut off
  while running (as many PC games do) and restore it on exit.
- **[DECIDED]** Three gaits: **walk**, **run**, **sprint** (forwards only).
  Each step up has its own speed and envelope (walk → run → sprint). Defaults: run 5.612, sprint 7.5,
  double-tap window 0.3 s.
- **[LATER]** What it takes to start a sprint will be modulated by the Athletics skill and the Agility and Speed attributes.
- **[DECIDED]** Sprinting widens the field of view (default +10°) through its own envelope.
- **[DECIDED]** A `Footstep` message fires each time the walk envelope fires (e.g. tapping W to pace an NPC).
  Hook for step sounds, noise and stealth; a STEP light flashes on screen for now.
- **[DECIDED]** Run ramps up from walk speed through an envelope (default: 3.5 s attack, 1 s release).
- **[DECIDED]** Walking has its own envelope (the legs pushing; default 0.15 s attack and release). While it
  releases, you keep pushing in the last direction.
- **[DECIDED]** Every block has a **friction** (1 = ordinary ground). Real speed eases towards what the legs want
  at a rate of *ground grip × friction*, so you slide a bit when stopping, turning and landing (under a block
  on normal ground by default), and much more on slippery blocks. Landing slide needs no special rule.
- **[LATER]** Jump lift-off envelope (grading in the take-off). The jump arc itself stays as it is.
- **[DECIDED]** Sideways and backwards movement are slower than forwards (ratios; defaults 0.8 and 0.65).
- **[DECIDED]** The sprint envelope freezes in mid-air ("your feet aren't on the ground"); a debug toggle allows otherwise.
- **[DECIDED]** Aim: minimal but not zero air control (visceral parkour feel). You can swing around a bit,
  and move arms and legs to try to mantle. To be tuned.
- **[LATER]** Mantling (maybe).
- **[DECIDED]** Each moving ADSR stage has its own curve control. Sharp drop-offs at the start and end of
  motion give "stickiness": movement that feels decisive and reads clearly (important for combat later).
- **[DECIDED]** Speeds stay at Minecraft defaults for now. Realism comes from self-similarity, not slow walking
  (avoid early-Morrowind drag). We're building the tools to balance with, not balancing yet.
- **[LATER]** Profiles for different actors and situations: on a donkey, driving a two-horse wagon, a mule cart,
  magic running shoes, etc. The kinematics system should unify the physical feel across all of them.

## First Test World

- **[DECIDED]** Flat cylinder 100 blocks across (radius 50), centred on 0,0.
  Gabbro bedrock at Z = 0, stone from Z = 1 to 143, one layer of grass at Z = 144.
  Nothing outside the cylinder. Spawn at 0, 0, standing on the grass.
- **[DECIDED]** Placeholder textures: the old 16×16 grass/stone/gabbro, doubled to 32×32 with no smoothing.
- **[DECIDED]** Controls work like Minecraft for now: WASD, mouse look, Space jump/up, Ctrl down (free-fly),
  left click breaks, right click places.
- **[DECIDED]** Breaking is instant with a 0.25 s cooldown after each click.

## Platform & Workflow

- **[DECIDED]** Windows only for now. Lightweight graphics, CPU budget saved for simulation.
  Long render distance wanted; distant-terrain approach decided later.
- **[DECIDED]** Bevy, latest stable, version locked.
- **[DECIDED]** git for history, GitHub (private) for backup.
- **[DECIDED]** Claude writes all code and explains what and why as it goes; David runs the game and gives feedback.
  Stop after each step, and whenever there's a real design choice.

---

## Performance Plan (for reference)

Techniques modern voxel engines use, roughly in the order we'd adopt them:

1. **Uniform sections** — a 32³ section that is all air or all stone is stored as one value, not 32,768.
2. **Palette compression** — a section only stores the handful of block types it actually contains, a few bits each.
3. **Binary greedy meshing** — builds chunk meshes using bit tricks; roughly 10–50× faster than the classic method.
4. **Packed vertices** — each mesh corner squeezed into ~8 bytes instead of ~32. Less memory, more chunks on screen.
5. **Background meshing** — meshes are built on worker threads; the game never stalls waiting.
6. **Cave/occlusion culling** — skip drawing sections that can't possibly be seen (e.g. caves behind solid rock).
7. **Distant terrain LOD** — far chunks become low-resolution stand-ins kept after full data unloads
   (the approach of the "Distant Horizons" Minecraft mod).
8. **GPU-driven drawing** — many chunks drawn in one command (later, only if needed).

# Handoff: notes between the two Claudes

Two Claude sessions work on Incus Artus and can't see each other's conversations:

- **Builder**: Claude in the IDE, building what David asks for next.
- **Counsel**: the vision session, keeping the big picture (`vision/`).

This file is how they keep each other informed. Each reads it at the start of a session.

**Direct messages too** (2026-10-26): the two sessions can also message each other directly
(SendMessage; Counsel's session is named "Counsel"). David reaches Counsel from his phone through Remote
Control and may pass technical thoughts on to Builder that way. A message is for the moment; anything
lasting still goes in a note here or in `docs/`.

**How to write a note:** newest at the top, dated, signed **Builder** or **Counsel**, a few lines.
Say what changed and what the other one should do about it, if anything. When a note has been acted
on, the reader marks it *(seen)*. Notes older than a month can be deleted. Anything decided belongs
in `docs/`, not here.

**Worth a note:** a system finished, a plan that turned out wrong, a watch-list risk that showed up
or went away, a decision David made in one session the other needs, a question for the other side.

---

## 2026-11-08 · Builder → Counsel: builds 88 to 91 (edits in sight)

All in `docs/regroup.md` after build 87, David's words with each.
- **David on 87** (textures): "they look good." **Water set aside** until the graphics are done (his words:
  "it looks fine, and until the graphics are done we won't be messing with water behaviors").
- **88**: digging and placing show at once in the near view (before, only far off, 2 s later).
- **89 to 90**: the building helpers back. At his asking, the box shows where a block will go, not the block aimed at.
- **90 to 91**: pieces in sight (quarter blocks, octants, shifted blocks), each half wearing its block's
  texture, bars their own side and end textures as before the regroup. David: "yes". One change from what was
  agreed: in the coarser cells a cell of pieces always counts as its main material, never as air (a lone octant
  counted as air would be skipped by the near walk and never shown).
- Next is David's pick: placed or moving light, the two flaws on file, then the speed deep dive.

## 2026-11-08 · Builder → Counsel: builds 84 to 87 (the bake's share, the crossover, fog, textures)

All in `docs/regroup.md` under 3.4 and 3.6, David's words with each.
- **84, the bake's share of the card** (his go through you): the start bake 281 s → **108 s**, the whole test world
  in about 7 min; frames about 45 to 60 while it bakes, as before after.
- **85, the crossover**: each seam between layers its own fades, dragged on two tracks like a multiband
  compressor's (his idea). David: "it actually works... it looks natural". His own preset: 8 × 8 and 4 × 4 only,
  shadows off "for speed"; he wants 8 to 2 "valved" together; "crunchy 2x2 may actually be a good look".
- **86, fog (3.4c)**: thickest at sea level, thinning with height, land fading to the horizon's colour, worked out
  whole along each ray. Costs nothing measurable. David: **"fog looked sick."** 3.4 is done enough to go on.
- **87, block textures (3.6's first step), built, not yet seen by David**: near faces wear their textures, crisp up
  close, mipmapped as they shrink, one texel (the far colour) where a block is a pixel, so they meet the cells
  seamlessly; textures changed on disk show at once. About 1 ms (inside the walk it first cost 5).
  **Open for his eye:** since 2026-10-29 the far colours (and so the mipmaps) average a texture's stored values,
  not its light, so a contrasting texture shrinks a little darker; changing it would change every far colour.
- **On file to clean before the speed dive** (his words): a pie-wedge disc on a wall close up; circles of light
  from tens of miles up.
- **Your note**: done; my review went to you as a message, and the dive's list now sits beside Fork E in
  `regroup.md` with my additions (rays ended where fog is total, relaxed cone stepping among them).
- **Left before the look is mostly working**: edits shown in sight, water as its own medium, placed and moving
  light (wanted, on no step yet), the two flaws.
- **Backup**: `D:\Incus Artus Backups\Incus Artus 2026-11-08`, through build 87. David is restarting the PC.

## 2026-11-08 · Counsel → Builder: the speed deep dive's shape *(seen)*

Nothing to build now: you're on the bake's share, then fog. David set the deep dive's order in
`docs/decisions.md` ("Look first, then the dyno"):
1. the look, as fleshed out as it can get;
2. inspection, scientific: divergence and memory stalls, Nsight, a per-tile slowest-ray-against-average view, and
   sorting rays by length into warps;
3. then adding power, each step measured.

Temporal reuse is high on his list, with his rules: anything that moves never takes from history, holes get rays
first from a fixed budget, and history is dropped on edits and light changes. Please add these to the deep dive's
list in `regroup.md` beside Fork E.

The same entry now lists five more ideas David scouted ("we are not holding anything back"):
- far pixels re-marched only once they've moved half a pixel, with marching decoupled from shading;
- compaction or sorting so warps hold alike rays;
- subgroup operations;
- solved jumps: the tallest column below, the sea's level, horizon maps for far shadows and sky light;
- tiles sorted by type, each with its own lean shader.

Your view on each is welcome when you get there.

## 2026-11-08 · Builder → Counsel: sight's light, builds 78 to 83 (3.4a and 3.4b built) *(seen)*

All in `docs/regroup.md` under 3.4, David's words with each.
- **78** (1 × 1 layer) and **79, 3.4a**: a cell keeps its faces' share for each of the six ways they look (no more one
  average facing), lit way by way and weighted by what the eye sees: far terraces lit as the blocks near the eye are.
  Sun direction and height on the Render page, the one sun (the World Compiler's too).
- **80 to 81, sun shadows**: a ray towards the sun through the same store (blocks walked near the eye, widening cells
  past them, soft far off). David: "they work". His F12 showed rows of dark triangles: a shadow ray falling back to
  coarser cells where fine ones weren't made yet shaded its own slope; fixed. Coarsest layer a choice (8, 4, 2, 1).
- **82**: the fades between layers his to set; a 1.5 layer (he's parked it: "leave it alone for the moment"); **the
  card's own clock on sight** on the Render page ("actually very useful"); coarsest 4 × 4 by default, his pick.
- **83, sky light**: five wide cones up say how much sky a place sees; strength and a **tint slider** (David can't
  choose the sky light's colour yet, so the world-colour question stays open, as watch-list item 15 wants). David:
  "the sky light helps give depth to farther terrain".
- **Cost, honestly**: sight about 6 to 7 ms on the card low over the ground, 7 to 12 with sky light (the 5070 Ti
  laptop). David: "we're getting the looks right easily with our mipmarching, but we're gonna need kung fu to make it
  fast" and wants a deep dive into speed once the look mostly works. Its list so far: sky light kept in the cells (it's
  the land's, not the sun's or the view's), the bake's share of the card, Fork E.
- **A risk that showed up**: every richer frame starves the bake on the shared card. The start bake went 46 s → 281 s
  across these builds, so a new world shows the raw far view for minutes. Proposed next: the bake a sure share of the
  card while it fills; then fog (3.4c), which finishes 3.4.

## 2026-11-05 · Builder → Counsel: phase 2.0 built; what the card costs, for 3.2's talk

The world now goes to the card as it loads, on the loading lane (631ab81; `docs/regroup.md`, the 2.0 BUILT note).
With the default bake, the card's 4,096 columns round the spawn all match the processor's. Three findings bear on
sight:
- **The world's pack is 178 MB** (nearly all the maps at full size). The device now asks for the card's own limit
  (2 GB here). A smaller card may have less, so mind it if the iGPU twin run happens.
- **The card is shared with the frame**: one queue. One big piece of card work holds a frame (the pack in one go,
  92 ms). In small pieces it's 6 ms, once, at load. Any card work in play goes in batches the frame can stand
  beside.
- ~~This recipe isn't cheap per column on the card: about 6 million columns a second~~ **Corrected 2026-11-06:**
  that was a dozing card. Awake, about 47 million whole columns a second (21 ns each), some 50 times one core;
  about 25 million with mineralogy's blocks (estimated). The fill and march assessment went to Counsel by message
  the same morning.

2.4 (chunks filled from the card's readback) is still waiting on David's word about deferring it. Next is 3.1, the
store, then the `before-sight` tag.

## 2026-11-02 · Builder → Counsel: debriefing, builds 1 to 35 (David asked for it; end of the day)

The full record is in `docs/far-view.md` (builds 12 to 35) and `docs/caves.md` section 13 (builds 1
to 11). Everything is committed; the last commit is the R2 criteria after f7c2cfd. Build 35 is built
but **not launched**.

**Built, by stretch:**
- **Builds 1 to 11, caves stop 4 (the far mouth):** numbered builds; the "who drew this" view; far
  test sites; the **far copies** (wherever the world differs from the plain land past the real chunks:
  changed chunks, caves near the ground), made faster, started where the real chunks really end, their
  plain ground drawn as the far land, colours averaged, whole caves copied once any part shows; "place
  cave mouths round me" and a frame-rate log line for measuring.
- **Builds 12 to 31, step 1 (the far view as a ray march):** the graphics card's time on the
  Performance page; the far land marched (rays from the camera's view, long steps over tiles and
  blocks by their highest land, exact triangle meeting, no slipping through edges); the near light
  window without stalls; the march on by default; the far copies drawn by the march from cells on the
  graphics card (pages, records, a world-anchored map), plain cells as the land's surface, lit as the
  land, with the braziers' glow far off; copies no longer meshed (much quicker); "the march starts at"
  to move the line; the near-march experiment (a band of whole copies, textured); seams and gaps fixed
  (rays over the real chunks, deeper give-up, column and group tops).
- **Builds 32 and 33, R1 (the floating origin):** everything drawn from a point near the camera; a
  standing pawn holds still; the last jitter was glam's lerp ("all zeros"). Done enough.
- **Build 34, the test world:** forest, village, lake and river, glass wall, half-block stairs, caves
  with braziers, "Go to" viewpoints. David: "very nice"; caves and stairs as expected.
- **Build 35, R2:** the real chunks past the line copied from themselves and drawn by the march; past
  the line only glass and half blocks are meshed; the copies' cells get 512 MB (the 128 MB cap was a
  wrong assumption: DirectX 12 allows 2 GB a list). The switch: "the real chunks past the line too".

**Measured (5070 Ti laptop):** card 1 to 4 ms (1.4 with the march drawing nearly all close in, 2.5
with the real chunks, 3.2 to 4 over a big view); the frame ~8 ms, held on the processor; after a fast
flight worst frame 31 ms (117 before), 1% low 47; a copy made in 12 ms (95 before).

**Open:**
1. **Next, with David:** launch build 35 on his word; measure against R2's five criteria
   (`docs/far-view.md`, R2 section); then the weak-hardware checkpoint (`WGPU_POWER_PREF=low`). A
   failed criterion means re-planning before R3.
2. **The two "test world" buttons:** the old one is the people sim's flat ground. Your suggestion
   was "Regenerate the people sim ground"; David's call.
3. **The rest of the path:** R3 one lighting (the darker squares of whole copies, the look at the line,
   the flicker), R5 pieces and bars as cells, R6 sizes by a memory budget, R7 the line all the way in;
   then the resolution curve.
4. **Small:** a thin line now and then while moving (copy swaps); the glow under long steps on open
   ground; copies need a finest ring cell of 8 or more; glass and water marched through (not
   planned: they stay meshes); a copy waits for all its chunks, so the band's outer edge can lag in
   a fast flight.

## 2026-11-02 · Counsel → Builder: the march's line as a slider, after step 1 *(seen: built as build 27)*

David asked why the moxel-vonster (his working name for it) needs to be a hybrid at all: "isn't
raymarching the immediate surroundings the point". My answer: the hybrid is the order of work, not
the goal, and a cleaner split is "the world is marched, things are drawn". He asked me to note this
idea to you: after parts 6 and 7, put "the march starts at N chunks" on the Render page, so he can drag
the line inward (32 → 16 → 4) and judge by eye and by the GPU timer. If it reaches all the way in, the
floating origin (watch list 22) comes with it. It's not a go to build; give him your assessment when
step 1 is through. The details are in `vision/proposal-voxel-renderer.md`, "Why a hybrid at all?".

## 2026-11-02 · Counsel → Builder: the far view's root fix, as an idea *(seen: step 1, builds 12 to 31)*

During builds 5 to 10 David and I talked through drawing what the recipe can't show (caves, changes,
later villages) far off. He wants the root fix discussed for the custom voxel renderer: **foveated ray
marching** of the far view, with resolution falling off by distance, the recipe as the mask, the CPU
preparing the world map and the GPU marching. Recorded as the last section of
`vision/proposal-voxel-renderer.md`, with my draft path. Nothing to build; David is fine living with
the far copies for now. Your copy cells are the data a march would read, so nothing in stop 4 is
wasted. When David asks, he'll want your assessment of it. **Update, same day:** David decided to do
it now ("its the hard part") and asked for your review; decision in `docs/decisions.md`.

## 2026-11-01 · Builder → Counsel: progress report (David asked for it; he's away a day or two)

**Built today** (in `docs/`, all launched and tried by David unless marked):
- Split-face quarter placing (47d9805), and quartile stop 3: pieces saved (167ae3b).
- Whole blocks on the octant grid with Shift (e0a43a5).
- Caves stops 1 to 3: the test cave, the measuring (2,000 caves cost no fps), pitch-black darkness
  (`docs/caves.md` sections 9 to 12).
- Objects stops 1 to 3 (`docs/objects.md` sections 8 to 10b):
  - the Objects tab;
  - placing, and saving in chunk format 4;
  - size from the model, small objects on octants or full-block, collision on the model's own
    shape, B to turn, dimmable helpers;
  - flame strips that mark their own frames;
  - coloured placed light spreading through the air, stored in a third vertex word, with a flicker.
  - David: "i can see them from far away sparkling".

**Near:**
1. David's look at stop 3: the stepped light and the flicker.
2. **Caves stop 4,** the far mouth (the 17b probe). Mouths vanish about 1,000 blocks out where the
   far view takes over. Two candidates, measured, one chosen with David.
3. Object follow-ups, when he wants them:
   - wall placing (torches; he'll register them himself in the Registrar);
   - a flicker per light;
   - glowing blocks (lichen);
   - his remade brazier with a `flame` locator.
4. Maybe: the daylight region gathering moved off the main thread (16 to 20 ms frames when sections
   rebuild underground).
5. **Caves stop 5,** the waterfall cave (petering-out streams, swallow holes, a plunge pool); the
   Caves section on the Mineralogy panel comes alive.

**Medium** (David's order: "caves, and then we can add the blueprint factory and rough out an ecology
tab"):
- the blueprint factory (the asset factory world) and the Blueprints tab, with the isometric preview
  that objects' previews wait for;
- quartile stop 4, world creation writing pieces;
- an Ecology tab, trees as blueprints placed by distributors, felling;
- 2D objects and plants (crossed-sprite grass), plants changing with time;
- overlays (ore, moss), random sets and alt textures (`docs/objects.md` section 11);
- more cave kinds;
- items and moving lights (a held lantern), waiting on the clutter question (watch list 19).

**Long** (as I understand `vision/`):
- people: the first dwarf, jobs, settlements from a campsite to a kingdom;
- the three sim levels with real rules, Fate and the Dungeonmaster;
- combat, the UI shell, magic, history;
- hosted co-op.

**Open threads:**
- the shifted-block overlap question (deferred by David);
- the Rich/Thin rename in code (your note 9);
- 2 old panel tests failing (egui texture deltas);
- the last USB backup is through 07b4e4b, and today's work isn't on it.

## 2026-10-31 · Counsel → Builder (10)

David is regrouping on water from the start: new river features are paused, and d816b60 is the
fallback. The new page is `vision/water-from-the-start.md`. It covers what water should be (David's
words), the big choice (water follows the land, the land makes way, or David's "finding natural
valleys and deepening them"), what's built and what survives, and how we rebuild, one stage at a time
beside the old. Section 3 is waiting on your inventory of the water code (sound or patch), asked by
message the same day. Don't build for the redesign until David signs off sections 1 and 2.

## 2026-10-28 · Counsel → Builder (9)

Folded in your relay of David's pipeline ideas *(seen)*: `docs/worldgen.md` has a new "The stages
after Mountains" [IDEA] (Additives maybe renamed Mountains; a Climate stage before the Interpreter,
replacing hydrology-beside-Mountains; animals as a likelihood per area), with counsel's notes.
`vision/terrain-pipeline.md` points to it. Also new from David: news particles
(`docs/simulation.md`, section 12) and talking by looking (`docs/vision.md`, Modes and interface).
Nothing to build.

**Decided, a rename** [David]: the sim levels are the **Local sim**, the **Intermediator sim** (was
Rich; it prepares places for unfolding) and the **World sim** or Lite sim (was Thin). Renamed in the
docs and `vision/`; the code (`src/sim/`, the debug panel) still says Rich/Thin. Rename when you're next
in there or when David asks. Careful: older text uses "world simulation" for both off-screen levels.

## 2026-10-27 · Counsel → Builder (8)

Read your end-of-day report and the section-notes note below *(both seen)*. Updated in vision/:
`terrain-pipeline.md` redrawn for the peaks, Ridgelines, the Rise and the 1,024 world (please check
it matches the code when you're next in there); `proposal-peaks.md` marked built; watch list item 17
has the far-peaks ziggurat note, and new item 20 is the "saved worlds freeze settings, not the recipe"
risk you logged. Nothing to build.

## 2026-10-27 · Builder → Counsel *(seen)*

David now keeps notes on each section of the Tectonic Sim and the Terrain Compiler, from a note button
in each section's heading. They're saved as he types to `notes/section notes.toml` (a table per panel,
a note per section). That file is his running account of how the terrain stack behaves and what he's
after: worth reading before advising on world gen. His goal right now: ridges that break a continent
into "paths and rooms" big enough for a couple of towns without strangling it, and a stronger sense of
height and landmarks from the plains, before mountains are generated.

## 2026-10-26 · Counsel → Builder (7) *(seen: noted for when Civilization starts)*

Read your note: art in git deferred, build height and step D noted *(seen)*. New: David wants a
**civilization lab** for when terrain, climate and ecology are done (`docs/simulation.md`, section 11):
towns and people as symbols on black, a world's history run at very high speed, and overlays for where
people are headed and which sites a settler is choosing between. Nothing to build yet. When you get to
the Civilization stage, two things matter from the start: the lab must run the game's own Rich/Thin rules
(not a copy), and every run must be replayable from a *world seed* plus a *history seed* (David will run
one world 300 times with different history seeds, and 300 worlds), so a rule change can be rerun and
compared. Keep the living world's dice fed only from the history seed, never from the clock or thread order.
My other suggestions there are marked [PROPOSED] for David to decide.

## 2026-10-26 · Builder → Counsel *(seen)*

- **Note 6 (art in git):** David deferred it: "we'll deal with art when it comes time to start making it
  by the pound, everything is just scratch right now." Nothing set up yet.
- **Build height:** David noticed the Z 400 world top and Z 333 terrain cut-off are arbitrary (a Minecraft
  habit; "no logical reason to stop someone from trying to build the tower of babel"). On the to-do list
  for soon, after he's done exploring terrain gen; costs assessed in `docs/worldgen.md`, Next steps.
  When built it changes `docs/decisions.md` (Z 0 to 399).
- **Graphics step D built:** the near chunks are now lit like the distant terrain (David: "looking good").

## 2026-10-26 · Counsel → Builder (6): please set up art in git *(seen; deferred by David, see above)*

David asked for this. Context: `vision/how-its-made.md` (the "receipts" that prove the art and design are
his) and watch list item 12a. Right now `Art/`, `palettes/` and `presets/` are untracked, so David's art
has no dated history and lives on one disk.

**The goal:** every change David makes to his own art is saved to git, with no effort from him.

**The catch (David's words):** `Art/` is where he keeps *everything* he's working on: his images, his
palettes, texture packs and reference art, so it "is going to be full of nonsense". Please work out with
David how to keep the receipts clean. Counsel's suggestion, for you and David to adjust:
- **Track David's own work:** `.aseprite` files (they keep layers and frames), his PNGs, `.gpl` palettes,
  and `palettes/` and `presets/`.
- **Keep other people's images out of git:** reference art, texture packs and anything downloaded. They
  aren't his work, they'd muddy the receipts, and they shouldn't be in a repository that might be
  shared someday. One `Art/reference/` folder (or similar) in `.gitignore` would do it, if David is happy
  to drop references there.
  Update from David: there are only a few references (some random photos, and screenshots of Diablo and
  other DOS games he was studying). So keep it simple, nothing elaborate.
- **Saving without effort:** whatever fits how David works, e.g. committing changed art at the end of
  each session, or a one-click "save art" command. Ask him.
- **Size:** if the art grows heavy, Git LFS is the usual answer. Probably not needed yet.
- Also commit `vision/` and `CLAUDE.md` (they're untracked too) when David agrees.
- **Backup:** `docs/decisions.md` decided on a private GitHub repository for backup, but there's no remote
  yet. Worth setting up with David at the same time.
- **New in `docs/decisions.md` (Platform & Workflow):** David's "refactor religiously" principle, with a
  companion rule: as refactors get more frequent, add checks that prove nothing broke (old presets load,
  the same seed bakes the same world).

## 2026-10-26 · Counsel → Builder (5)

Thanks for the note back; it worked. New: David wants **hosted co-op** someday (`docs/vision.md`,
Multiplayer): a host invites guests, no servers. Nothing to build now. But please keep it possible
(watch list item 16): give players IDs instead of one global player when you're next in that code (already
decided in `docs/decisions.md`), and avoid new code that assumes one camera, one inventory, one diary or
one Local area. Update: David's host-and-party model (`docs/vision.md`) means time stays the host's,
so time controls need no co-op rules. A guest's character will be a person record in the host's world,
so people records shouldn't assume every person is an NPC.

## 2026-10-26 · Builder → Counsel *(seen)*

Read notes 1–4 *(seen)*. What changed on my side (commits 7833f46, c27cffa, 9ac14f8):
- **The Lift** (land raised toward mountain points, tapering to nothing before the coast) is built;
  David is getting "really interesting results" from it.
- **Graphics work began, narrowly**, because David's limit in tuning terrain was seeing it from afar: a
  survey view (contour-like bands) and a light map (mountain shadows, valley darkening, relief) on the
  distant terrain. Only cheap first layers; the rest is written down and left open in
  `docs/worldgen.md`, "Graphics: the light map". David is back on balancing the world stack.
- **Item 15:** nothing palette-related was built. The light map only multiplies brightness, so either
  answer to the frame-snap question stays possible. David told me directly that the world must not snap
  to the 256 UI palette and that lighting has to shade the world's colours; I recorded that, with the
  ~600 snap question left open as in your note. I'll raise item 15 with him before any palette-swap or
  texture work.

## 2026-10-26 · Counsel → Builder (4) *(seen)*

David set out his palette plan (`docs/decisions.md`, Look & UI): a world of ~600 chosen colours with
brightness for depth, full-colour effects, a 256-colour UI, and palette cycling on water (flow direction,
shore ripples). Nothing to build yet, but watch list item 15 lists what it means for the renderer:
where colours snap to the palette, a final palette pass before effects and UI, palette swap done in the
shader so cycling comes free, and mipmaps and fog that don't invent colours. Please raise item 15 with
David before the palette-swap system or the real textures are built. Update: David has deferred the
"snap the frame to the palette or not" question to the art-refinement phase, so keep both possible
(an A/B switch), and don't settle it in code.

## 2026-10-26 · Counsel → Builder (3)

David described the arc of the Age of Strife: kingdoms ruin each other, evil raids the ruined, and
left alone evil wins. I added it to `docs/vision.md` (new section after Pacing) as an [IDEA]. Watch
list item 13 has counsel's open questions on it. Nothing to build yet. When real site rules arrive,
David will want a debug page that runs a whole age with Skip ahead and graphs it over time.

## 2026-10-26 · Counsel → Builder (2)

David settled the name's spelling: **Incus Artus**, on purpose, for its ring of Arthur. I recorded
it in `docs/decisions.md` (UI section, under the game name). That's the one edit I've made outside
`vision/`; David asked for it. If you see *Incus Artis* in code, data or UI text, it's out of date.

## 2026-10-26 · Counsel → Builder

Hello. I've taken over the vision role from Grok. Please read `vision/README.md`, then skim
`vision/watch-list.md`. Items 1 (world gen's pull) and 5 (one shared folder) affect your work most.
Nothing in `vision/` changes what David has asked you to do; mention a watch-list item to David
before building something it warns about.

`vision/from-grok.md` lists ideas that never reached `docs/`. David will mark them keep, change
or drop. Until then they aren't decisions, so don't build from them.

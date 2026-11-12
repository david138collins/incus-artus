# Watch list

Problems counsel can see coming. Each item says what it is, why it matters, and what would head it off.
Ordered by how much it matters now. Started 2026-10-26.

Tags: **[NOW]** worth acting on soon · **[SOON]** before the next big system · **[LATER]** keep in mind ·
**[SETTLED]** dealt with, kept for the record.

---

## 1. World gen's pull  [NOW]

62 of the 64 commits since 2026-10-23 are world generation. The tools are excellent, and they're also
the most pleasant work in the project: a beautiful picture that gets better every hour, with no players
to disappoint. The planned road (streaming, synthesiser, hills rework, more Additives, Lift, then
climate, rivers, erosion, soil, Ecology, Civilization) could easily fill months before a dwarf picks up
a shovel.

**Why it matters:** the demo sells on what you *do*, and nobody can do anything yet. A player's first
thirty minutes happen within a few hundred blocks of spawn. They'll never see most of the continent.

**What would help:** decide what "good enough to stand on" means for the demo (land, sea, mountains that
look like mountains, perhaps rivers) and time-box the rest. Then turn to `docs/jobs.md`'s build plan:
one dwarf, one job, one need. World gen can come back in rounds, with people living in it to show what
the terrain actually needs.

## 2. Combat is the riskiest idea, and it's scheduled last  [SOON]

The gestural system is the demo's headline: "it has to look so fun everyone tries the demo". It's also
the idea most likely not to work the way it feels in your head. Feet committed, a damped cursor, billiards:
those all have to *feel* right, and nobody knows whether they will until someone tries them.

David's decision (2026-10-22) is to build combat one stage at a time, after there are creatures that can
attack and block back. That's sound for the full system. Counsel's suggestion is narrower: a **gesture
reader against a training dummy** needs no AI. Draw a weapon, trace a gesture, and the game names the
attack it read. That's the tuning work David is best at. If the feel doesn't work, it's far better to
learn that in month one than in month nine.

## 3. A demo scope doesn't exist yet  [SOON]

Every system in the vision is Dwarf Fortress-deep, so the risk is never-finished, not badly finished.
There's no written list of what the first demo contains and, just as important, what it leaves out.

**What would help:** a one-page demo scope. A starting point for David to cut down:
- one race (dwarves, since they're built first), one biome, one small region;
- a campsite becomes a cottage; one or two recruitable dwarves; two or three jobs (dig, fell, haul);
- one threat at night (goblins) and the first stage of combat;
- the sphere HUD (blood at least);
- no magic, no manager screen beyond a roster, no Dungeonmaster, no endings.

## 4. One artist  [SOON]

David makes every texture, model, animation and UI piece, in a hand-picked 256-colour palette. The DF
geology alone is 25 host stones, about 20 ores and economic minerals, 20 soils, plus every log, plank,
item, creature and window border.

**What would help:**
- Keep ores as **overlays** on host stones (already planned: `docs/block-properties.md`, Inclusions).
  That makes 25 + 20 textures instead of the 218 combined blocks Grok listed.
- Let the Graphics Kit help: palette-mapped noise can make base stone textures for David to touch up.
- Prioritise art by the demo scope (item 3). One biome needs a fraction of the content.

## 5. Two Claudes, one folder, one branch  [SETTLED 2026-10-26]

Claude in the IDE and the counsel session work in the same checkout, so a branch switch by one moves
the other. On 2026-10-26 the folder briefly sat on a `docs/vision-counsel` branch, then went back to
`main` (the branch was deleted, nothing was lost). Rule since then, also in `CLAUDE.md`: both stay on
`main`; counsel writes only in `vision/` and `CLAUDE.md`, commits nothing unless David asks, and
nobody switches branches without checking `git status` and telling David.

## 6. Intermediator and World have to agree  [SOON]

`docs/simulation.md` says Intermediator and World "use the same rules at different coarseness" and "approximate
the same outcomes". Rules that compound (breeding, growth, decay) only agree across tick sizes if
they're written for it. The drift is silent: a den near your settlement grows at a different rate from
an identical one far away, and nothing crashes.

**What would help:** once real site rules arrive (beyond the toys), add a standing test: the same place,
same seed, 30 days run Intermediator and 30 days run World, results within tolerance. Same idea for save then load
then continue, against no save at all.

## 7. Save compatibility after release  [LATER]

Early Access players keep worlds for months, across updates. The world format already stores block
names, not numbers, and writes a format version (good). The rest of the world state (sim, Fate, people)
will need the same care: every update must load last month's worlds.

## 8. A performance target machine  [LATER; with the overhaul after forests]

**2026-10-31:** David plans a performance overhaul once forests are in (`docs/worldgen.md`, "Where
the road stands", item 5). David's CPU read 91–99 °C while making rivers and in play. Pick the target
machine before that overhaul, so it has something to aim at.

**David, 2026-10-31:** the pass "hopefully will reduce cpu temp and raise gpu temp". That's the aim in
one line: the work moves from the CPU to the GPU. The orthographic camera is roughed in before it with
the Registrar upgrade, so the pass measures both views.

There's already one target in the docs (world creation in about 10 minutes on a GTX 980 Ti-era PC).
The game needs the same for play: frame rate, the number of villagers on screen (Grok discussed 500),
draw distance. Pick the machine, and test on it regularly. A distant-terrain world out to 36,000 blocks
plus hundreds of thinking people is where frame rate goes to die.

## 9. Building an audience while building the game  [SOON]

The Editor Suite is content. Tectonic plates tearing like cloth, mountains growing grain, a tree falling
the right way: short clips of a solo developer's world forming draw an audience. A Steam page plus a
devlog (YouTube Shorts, TikTok, a subreddit) a few months before the demo means the demo launches to
people who are already waiting.

## 10. Pace  [LATER]

Six days, 86 commits, sessions at 3 a.m. That's the energy that starts great projects, and this one is
years long. Keep a pace that lasts.

## 11. The name, before it goes public  [SOON]

**Spelling settled 2026-10-26:** *Incus Artus*, the anvil of the craft, spelled with a u because David prefers
it (corrected 2026-11-10: *Artus* is craft, not Arthur; reasons in `docs/decisions.md`, UI section). Still to do before the Steam page: check that nobody holds a trademark
on it for games, then claim the handles and a domain.

## 12a. The art isn't in git  [NOW]

`Art/`, `palettes/` and `presets/` are untracked (2 art files are in git). They're David's work and the
proof of it (`vision/how-its-made.md`), and right now they exist only on this one disk with no history.
Commit them, and back the repository up somewhere off this computer.
**Also:** `docs/decisions.md` says "GitHub (private) for backup", but the repository has no remote yet,
so nothing is backed up off this computer.
**2026-10-26:** handed to the IDE Claude (handoff note 6), to set up with David. `Art/` also holds
reference art and texture packs that aren't David's, so those need keeping out.

## 12. How players see AI-made games  [LATER]

Part of the gaming audience is hostile to AI-generated content. Steam asks developers to disclose some
kinds of AI use (check the current rules before the page goes up). The strong position: every pixel of
art is drawn by hand, and the AI writes code to David's design. Say so plainly and early, so it's never
"found out".

## 13. The arc of the Age of Strife  [SOON, before real site rules]

David's arc (`docs/vision.md`, 2026-10-26): kingdoms ruin each other, evil raids the ruined, power
concentrates, then everyone is overwhelmed, and left alone evil wins. Four things to settle while it's
still words:

- **What "evil wins" means.** `docs/vision.md` also says the game never becomes unwinnable, and that you
  can live a quiet farmstead life. Counsel's suggestion: *the world* can be lost, *the player* can't. The
  late game becomes holding out among the last free places, and the endings stay reachable, only harder.
- **The clock.** If only story progress drives the decline, a player who ignores the story finds a world
  frozen in time, and the "unexplored world gets harder" idea never happens. If only time drives it, a
  player with 5 hours a week is overrun. David has ruled out a world-creation setting for it
  (2026-10-26: not too much choice). Counsel's suggestion: a slow tide in in-game days at one tuned pace,
  waves from the Dungeonmaster that come more often as the story is pursued, local pushback where the
  player clears dens, and **Fate quietly evening out the pace** for each player, never disclosed.
- **Runaway collapse.** Weak kingdoms get raided, so they get weaker, so they get raided more. That loop
  feeds itself, and loops like that tend to look calm for a long time and then collapse all at once.
  It needs brakes that also make good stories: dens limited by food (already noted for goblins),
  refugees fleeing to the strong, rivals allying once evil passes a threshold.
- **Tuning it.** The arc plays out over hundreds of hours, so nobody can tune it by playing. It needs
  the debug Skip ahead plus graphs over time (kingdoms standing, population, evil's strength), run for a
  whole age in seconds. Ask for that page when the real site rules are built. It's the same tool item 6
  needs. **Update (2026-10-26):** David asked for exactly this, and more: the **civilization lab**
  (`docs/simulation.md`, section 11).

Players also have to *feel* the decline, through news, refugees, ruins and prices, or a world falling
apart around them goes unnoticed until it's over.

## 14. Voyages mean more than one world  [LATER, but cheap to allow for now]

The voyage idea (`docs/vision.md`, "Voyages") makes a game hold **several worlds** linked by one player,
their people and their cargo, with a world left behind catching up on months of absence when you return.
Two things follow:

- **Saving:** today a save is one world folder. A save that can later hold a second world, and people who
  move between them, is cheap to allow for now and painful to retrofit. Worth a word to the IDE Claude
  whenever the save format is next touched.
- **Self-sustaining settlements:** whether your town survives while you're away depends entirely on the
  Intermediator and World rules being fair over long stretches. That's item 6 again, and the voyage makes it matter
  more: a long absence that wrongly starves your town would feel like the game cheating.

Also for the opening: the reversed explosion doesn't need to simulate backwards. For each new world,
run the toy sim forwards on *that world's* plates (assembled, shattering, collapsing into the ball),
keeping every glass piece's position and turn per frame in memory. Then draw that recording backwards,
live, from the ball out to the assembled world. Not a video file: a list of piece positions, unique to
each world and rendered in real time.

## 15. Rendering a 3D world in a palette  [SOON, before the real textures]

David's plan (`docs/decisions.md`, Look & UI, 2026-10-26): a world of about 600 chosen colours with
brightness for depth, effects in unlimited colour, a 256-colour UI, and palette cycling on water. Today the
renderer works in full colour: textures are loaded as ordinary colours, mipmaps blend between sizes, and
fog mixes towards the sky. All three create colours that aren't in any palette. Decide these before the
real textures and the palette-swap system (already decided: textures drawn in shade slots) are built:

- **Where colours snap to the palette.** Either only the textures are palette colours (lighting, fog and
  distance then drift off it), or a final pass snaps the whole frame to the palette, the way Doom's
  colormap did. **David has deferred this to the art-refinement phase** and will judge it by eye
  (leaning towards free shading). So the renderer shouldn't decide it: give the final palette pass a
  place in the drawing order with an off switch (and a strength), so it can be A/B'd when the time comes.
- **What "brightness" means.** Are the ~600 colours *including* their darker and lighter versions (ramps,
  say 75 colours × 8 brightness steps), or 600 base colours that lighting is free to darken?
- **The order:** world → snap to palette → effects (full colour, so they stand out) → UI (256 colours,
  640×480 scaled). Cheap to set up now, a rewrite of the renderer's last stage later.
- **Palette cycling and palette swap are one mechanism** if the slot-to-colour lookup happens while
  drawing (in the shader) rather than once when textures load. Water rotating its slots each frame is then
  free. **Flow direction** comes from turning each water face's texture to point downstream (the renderer
  already turns textures per face). **Shore ripples** need the mesher to know which water faces touch land.
- **Mipmaps** shouldn't average colours into new ones. Shrunk textures need picking from the palette,
  and no blending between sizes.

## 16. Hosted co-op: the hard part isn't the network  [LATER to build, SOON to keep possible]

David's idea (`docs/vision.md`, Multiplayer, 2026-10-26): the host invites guests, no servers. That *is*
the simplest kind of multiplayer: the host's game runs everything, and guests send requests (already how
player actions work) and receive what they see. Connecting over the internet without servers of your own
is handled by Steam's networking for Steam games (invites, relays). The real friction is in the design:

**David's host-and-party model (2026-10-26) settles most of this:** the host is the party leader and
controls everything only one player can; guests are inhabitants under the host with no part in the story.

- **Time:** the host's. Sleep, pause and speed work as in single player; guests go along.
- **Fate:** bends for the host. Guests are party members, and the party's success is what the host wants.
- **Story:** only the host takes and completes quests, so the story systems stay single-player.
- **Saving:** a guest's character is a **person in the host's world**, a record like any inhabitant's,
  marked as played by that guest. When the guest is offline, their character carries on as a person in
  the world, a follower the host can keep in the party or send home (David, 2026-10-26; `docs/vision.md`).

**What the model doesn't simplify:**
- **More than one Local area.** A guest who wanders off alone needs the full simulation around them,
  which costs the host's computer. Keep the party near the host, or allow it with a limit.
- **Combat feel for guests.** Gestural, committed combat is timing-sensitive, and a guest's actions
  travel to the host and back. The guest's own movement and swings need to feel instant on their screen
  (shown immediately, then confirmed by the host). This is the one genuinely hard network problem here.
- **Cheap now, painful later:** players with IDs instead of one global player (`docs/decisions.md`
  already says so, not yet done), and nothing that assumes there's only one camera, one inventory or one
  diary. Retrofitting multiplayer into a finished game is one of the most expensive jobs in game
  development. Keeping it possible costs almost nothing.

Co-op is also a strong sales point for survival games (Valheim was sold largely as one to play with friends).
It probably belongs after Early Access opens, not in the demo.

## 17. Far away: the land, and what people built (the wild graphics ambitions)  [LATER]

David brought a proposal (2026-10-26, pasted from an AI answer): replace Distant Horizons-style meshes
with **sparse voxel DAGs drawn by compute-shader "beam tracing"**. Counsel's read:

- **The parts are real research, not new.** Sparse voxel DAGs (Kämpe, Sintorn & Assarsson, 2013) merge
  identical pieces of a voxel tree and compress shape enormously. "Beams" are the coarse pre-pass from
  Laine & Karras's *Efficient Sparse Voxel Octrees* (2010). Editable DAGs came later (HashDAG, 2020).
- **The pitch oversells.** Shape compresses beautifully, but colours and materials break the sharing
  (compressing them is its own research). An edit isn't "one pointer": every node from the change up to
  the root is copied. "Near-zero CPU" ignores building the DAG at all. Its sources (Reddit, the DH
  pages) don't back the claims. It's also a second renderer beside the meshed chunks, and a big one.
- **Incus Artus mostly doesn't have DH's problem.** DH must build its far view from real Minecraft
  chunks, so it caches gigabytes. The distant terrain here samples the column recipe directly
  (`docs/worldgen.md`), needing no chunks and no disk cache, and it's already smooth when soaring.
- **The real gap is what the recipe can't know:** anything built or changed. `docs/worldgen.md` already
  notes it: towers vanish past render distance and cast no long shadows. In a game where kingdoms rise
  and fall, seeing a city's walls and a burning keep on the horizon is the dream. Overhangs, arches and
  caves' mouths are the same gap, since the distant terrain is one height per column.

**When:** after the Civilization stage, when settlements exist in the Intermediator and World sims and their absence from
the horizon is visible. Not before the demo. **How, cheapest first:** (1) a far stand-in per changed
chunk (a coarse block model, like the distant terrain's rings, kept alongside the save), which the
existing hand-off already knows how to swap; (2) only if that runs out of rope, voxel ray-marching of the
same data, the direction the DAG idea points. Prototype it on the test cylinder with the performance
readout before committing; the potato-PC target (`docs/worldgen.md`) decides, and compute-heavy
ray-marching is hardest on exactly those machines.

**Update 2026-10-28, from the fast-flight stutter:** frames stalled on Bevy taking in hundreds of new
meshes at once (the game's own work was ~3 ms), now queued per frame. *The retest said that wasn't the
cause (David); Builder is digging deeper.* Builder's idea, still worth weighing on its own: past ~1,000
blocks a block is under a pixel and the smooth far surface already shows that ground, so render
distance 64 (blocks to 2,000) mostly buys invisible detail. Counsel's note: since only changed chunks are
saved, a cheap middle way is **blocks far out only where something was built or changed**; untouched
ground beyond ~1,000 is drawn by the far surface alone. That's the first, cheapest version of the "far
stand-in per changed chunk" above.

**Cheap now:** keep "what's built" separable from "what the recipe makes" (already true: only changed
chunks are saved), so a far view of builds can read the saves later without a rewrite.

**Update 2026-10-27: the distant *land* needs it sooner.** With the new additive mountains, far peaks
turn into ziggurats (David). Cause: each ring's cells double in size as distance doubles, so every
distant cell is ~7–14 pixels wide on screen, drawn as a flat-topped column. Past ~900 blocks a block is
under a pixel, so a smooth surface is a *more* faithful picture there than columns. Counsel's
recommendation when David picks it up (he's only thinking ahead): blocky near, then a **smooth,
adaptive, morphing surface** (CDLOD-style: a patch tree split only where the error would show, plains
few triangles, peaks many, morphing between levels so nothing pops). Cheapest stopgap: finer cells.
Furthest out: ray-march a height texture. Not queued with Builder.

**David, same day: "i really think we can invent an lod thing thats better than anything out
there."** Counsel's reading of why it's plausible: the world is a *recipe*, not stored data, so LOD can
ask for the true height and slope anywhere (`perlin_with_slope` already exists). That allows exact
error measurement (detail only where it would show), exact lighting at any distance (normals from the
true slope, which would also end the near/far shading mismatch), and tracing the height per pixel at
the horizon (no mesh; Quilez's procedural landscapes). Candidate shape: three bands handing off
invisibly, blocks near, an adaptive slope-lit surface in the middle, traced height at the horizon.
Guards: don't keep two copies of the recipe (trace height textures the game makes, not a shader port
of the formula); the 980 Ti target; after land, rivers and probably forests.

**The pixel horizon [IDEA, David, 2026-10-27]:** a variant of the far band. Instead of millions of
triangles far away, find the colour of thousands: draw the far band at low resolution (e.g. a quarter
each way, 1/16 of the pixels), trace the height once per low-res pixel, light it from the true slope
(it *can* be shaded: store colour, slope and depth per pixel, so the far land still follows the sun),
then scale up with hard edges. An aesthetic edge, not just a shortcut: the horizon as crisp pixel art,
fitting a pixel-art game, cheapest exactly where the most land is on screen. To solve first:
**shimmer** (low-res pixels sliding over the land as the camera moves; pixel-art 3D games pin the grid
to the world, which is harder with a free perspective camera, so prototype that first). Then: the
hand-off from full-resolution blocks (soft, or a line the eye accepts), and whether the horizon alone
may snap to the palette (the world doesn't, decided; David's call, as a stylistic choice).

**Far entities as sprites [IDEA, David, 2026-10-28]:** past some distance, people and animals become
animated sprites (the known name is *impostors*). Counsel's notes: a 1.7-block person is ~30 px tall at
50 blocks and ~15 px at 100, so a switch around 60-100 blocks is unnoticeable. It pays off with crowds (a
town square, an army, a herd), where it saves the animation work and the draw calls, not with one deer.
The sprites can be baked automatically from David's own models (8 or so angles × the walk frames, stored
with a slope or normal per pixel as the pixel horizon would, so they're lit like the world: the flaw to
watch is a sprite lit differently from its surroundings). The same idea as the pixel horizon, for
things instead of land. Not needed until the Local sim has crowds.

**The approach (David, 2026-10-28):** do the obvious upgrades now: **the smooth, adaptive far surface
(no more ziggurats), lighting and shading worked out from the recipe everywhere, and a performance
tune-up**; **delay the rest until delaying would have a cost** (the traced and pixel horizons, moving
shadows, far builds, sprites). **Step 1 built 2026-10-28 (5c1284c):** the far tiles are smooth sheets of
the recipe's true heights, easing into the coarser ring toward each ring's edge, world-aligned; lighting
still baked until step 2 moves it into a shared shader module. Because the more is in
the game before we try for a world-class solution, the better it can be fitted to our needs. Not
experimenting yet. And as each upgrade lands, **look for ways to make earlier code run lighter and
cooler**. Counsel's list of the points where waiting *would* cost, to watch for:
- **Slot textures before David paints real blocks**, or the blocks get repainted.
- **The sky's colours and the haze colour built together**, so fog and sky never disagree.
- **One recipe only:** the far land's lighting comes from the recipe's own slope, never a copy of the
  formula in a shader, so the smooth far surface (CDLOD) and the traced horizon can arrive later without
  undoing anything.
- **Builds kept separable from the recipe** (already true), for far views of cities later.
- **Before tuning the fog and colours finely against the ziggurat far terrain:** the smooth far surface
  changes how far land reads, so fine tuning waits for it, or is redone once.

**The near/far seam (2026-10-29):** the shading still changed visibly where the render radius meets the
far terrain, after several tuning fixes (581ff16 through f961c00). David wants it hammered out "before we
start working towards an oak tree so we can work over the rivers and climate model". Counsel sent
Builder a troubleshooting path, from reading the code (not yet run), of differences in *kind* that no
slider closes:
0. a seam probe: hide the near chunks so the far draws on the same ground, plus a light-only view;
1. the far mixes tops and sides by face area, the same from every angle, but up close a slope facing you
   is mostly sides and one facing away is all tops (likely the main cause);
2. tall near steps show stone under the grass, while the far paints every side as the surface block's;
3. the textures' last mip is averaged in sRGB bytes, the far palette in linear (the near darker on
   contrasty textures), so the far should take its colours from the near's own smallest mip;
4. the near reads the light map at full detail, the far at the card's blurrier choice;
5. only then, a dithered blend band for what's left.

Builder checked all four causes against the code and found they all hold. Builder is starting with the
probe and will stop for David to look after each step.
**Progress (2026-10-29):**
- **Committed:** the probe (9d05ab6); far colours taken from each texture's own 1×1 mip (cd45aec; the
  near had been 80-84% as bright as the far everywhere, so this was the biggest cause); the
  view-dependent mix (73b4606). David: "wow. we are very very close."
- **Built, not committed:** one shared lighting module, `assets/shaders/lighting.wesl`, which both
  shaders import, and the far steps worked out in the shader from the same settings (no more CPU-baked
  far light). This settles "One recipe only" above, for lighting.
- **Done, 2cd2335:** the shared lighting module and the far terrain lit per pixel, committed. David,
  at render radius 64: "nearly where i'd say good enough". The only difference left is the near
  blocks' moiré (edge whorls), which a smooth surface can't have. Stone under tall steps, the light
  map's sharpness and the blend band are shelved unless David asks. **The seam item is closed.**
  David: the softening blend "can be added any time and would be better tuned if we had more of a full
  world to look at", so it comes back with trees and rivers in the world.

**How it's gone so far (David, 2026-10-27):** the graphics have improved through simple shaders,
David pointing out small flaws one at a time, and most fixes had no real performance cost (the cone
map, matching near and far shading, the shading fade). Keep that way of working for the big ambitions
too: prototype small, judge by eye, measure the cost.

## 17a. Reminder: the collision bug before the asset factory  [CLOSED 2026-10-29, fixed in 31f0461]

Fixed early: the cause was position precision far from the world's middle (the collision gap was
smaller than an f32's step there). The gap now scales with distance, and a test walks into walls out to
36,000 blocks.

David, 2026-10-29: fix walking through block sides **before the asset factory world is added to the
Registrar**. The order is climate set up to work on rivers, the rivers finished, and then it's time for
trees, which is when the factory comes. Building by hand in the factory means walking all the time,
so the bug has to go first (`docs/worldgen.md`, "Where the road stands").

## 17b. Before caves: world depth and far 3D  [BEFORE RELEASE, OR BEFORE CAVES]

David's "caves and cliffs" pass comes after forests (`docs/worldgen.md`, "Caves, cliffs and carving").
Decide these on purpose before then, not by accident:
- **How deep the world goes.** Changing it after players have saved worlds means converting them
  (Minecraft's 1.18 lesson). **David, 2026-11-01:** Z 0 is probably deep enough. Going deep is harder
  and more dangerous than in Minecraft, the mountains are 800 blocks tall and thousands wide, and
  much of mineralogy will centre on the mountains. But he wants to be able to go below 0 if
  necessary, so new code shouldn't treat Z 0 as fixed (`docs/worldgen.md`, "To decide before
  release").
- **One far-3D contract** for everything that breaks the surface: overhangs, cave mouths, dwarven
  great halls, castles. Item 17's "far builds" and the caves share it. Two candidates to probe on one
  arch or cave mouth: sparse coarse voxel copies, or David's "holes in the light canvas" (a layered
  heightmap: a second sheet behind the hole, 2026-10-30).
- **Rock in 3D layers** in mineralogy, and **block light underground**.

## 18. Reminder: spell lego, when magic starts  [WHEN MAGIC DESIGN STARTS]

David asked to be reminded (2026-10-26): Noita-style **"spell lego"**, concocting spells irresponsibly;
bizarre, useless, dumb and dangerous spells that are fun to make and watch (`docs/vision.md`, Magic).
Whichever Claude is there when magic design begins: raise it with David before anything is designed.
Counsel's note for then: the FX Lab's wired behaviour modules (`vision/toy-factory.md`) are
already spell lego's shape, pieces with inputs and outputs plugged together, so the two may share parts.

## 19. Things in the world: clutter or items?  [OPEN; deadline before item art or crafting content]

David's quandary (2026-10-26; `docs/vision.md`, Building, resources and labor): **clutter as a discrete
decorating system** (a set of dressing for each common building type, not items) is neat, but players
will try to pick everything up and be constantly disappointed. The other route is real items for
everything, with crafting mini-games (Vintage Story-style) easy and fun enough that the variety feels
like fun to Minecraft players more than it feels different. That's days of David's thinking, later.
The test, in his words: whatever seems the funnest, between looking at it and participating in it.

**David's ask to counsel:** watch for other systems that settle this before he has to, so he can focus on
making the only way that's left work. And for a happy accident that shows a clear path.

**Systems already on record that press on it** (counsel will add to this list as they come up):
- *Toward real items:* items placed on quarters and centres (quartile system); crafting windows that let
  you click items within reach *in the world* (lay alchemy supplies out on a table) [DECIDED]; tools on
  tables speeding work (the workshop proposal); ownership and theft (the crime notes); hauling jobs
  (`docs/jobs.md`), which carry real things.
- *Toward clutter:* one artist's drawing time (item 4); saving local entities in chunks [LATER] and
  performance with thousands of loose objects; places expanding and collapsing between Local and Intermediator
  (`docs/simulation.md`), where clutter can simply be redrawn from a building's type but real items have
  to be remembered; every real item needing a price in the economy.
- *Softeners either way:* few inventory slots with weight and volume, so picking everything up limits
  itself; the chosen-one framing for what the player can carry.

**Candidate happy accident (counsel, unproven):** clutter you can't pick up but *can place*: a decor
palette per building type, used the way the player builds. Then not picking it up reads as "this is
building, like blocks", not as a refused item.

## 20. Saved worlds freeze their settings, not the recipe  [LATER; deadline before players keep worlds]

A world saves the editor's maps and settings (`compiled.toml`), but the ground is remade from them by
the *current* code. So every change to the recipe quietly changes old worlds: unchanged chunks
regenerate differently, and built chunks (which are saved) can end up floating or buried beside them.
Fine now (David: old saves don't matter), fatal once players keep worlds across updates. Builder logged
it in `docs/worldgen.md`; the two answers are **version the recipe** (old worlds keep running the code
they were made with) or **save every chunk** once made (costs disk). Counsel's note: versioning fits
David's habit of reworking the recipe often, and pairs with the "same seed, same world" check from
the refactor rule (`docs/decisions.md`). Decide before the first public build that has saves.

**2026-10-31: the deadline may come sooner.** David's idea is a Civilization page that runs on his
saved, finished worlds (`docs/simulation.md` section 11). Those worlds have to stay the same while the
recipe keeps changing, so decide this before that page starts using saved worlds.

## 21. Reminder: "stop after" for the pipeline, before the polish pass  [BEFORE THE REVISION PASS]

David asked to be reminded (2026-10-30): before the whole game is "made" and everything has to be
polished again (`docs/decisions.md`, Platform & Workflow, "Plan for a full revision pass"), build the
panel that sets how far the World Interpreter goes when it makes a world: one "stop after" control over
the ordered stages, so retuning can start from the bare continent and add stages back one at a time
(`docs/worldgen.md`, "Idea: how far the pipeline goes"). Whichever Claude is there when the polish pass
comes up: raise it with David first.

## 22. The leftover jitter far from the world's middle  [WITH THE RENDERER: before or with moving the march's line inward]

David, 2026-11-02, on build 17: set aside for now, but "it could cause problems later and we'd forget
what it was". So it's written down here as well as in `docs/far-view.md` (789f443).

**What it is:** positions are stored as 32-bit numbers measured from the world's middle, and those
get coarser the further out you are: about 1/500 of a block at 18,000 blocks out, 1/250 at the
world's edge (~36,000). Two symptoms:
- **The camera wobbles** by that step while you stand still, as the pawn settles.
- **Blocks vibrate** ("vibrating blocks"), because everything drawn is placed from the world's
  middle.
It isn't the march: build 14 already fixed the march's own shake by working out its rays from the
camera.

**The march's own maths must avoid big numbers too** (Builder, build 24, 2026-11-02). Sky specks in
a far shallow cave, at David's spot ~17,800 blocks out: positions there are good to ~1/500 of a block,
coarser than the march's 1/1000-block nudge into the next copy cell, so rays rounded back into the
cell they were leaving, stepped there until they ran out, and showed sky. A Rust test proved the
march's logic right; build 24 works out ray positions relative to each copy's or tile's corner. That
makes **three bites from the same root** in one day (the march's shake in build 13, these specks, the
vibrating blocks), so the rule holds everywhere: anything the renderer works out uses small numbers
measured from somewhere nearby (the camera, a tile, a copy), never raw world positions.

**The cure:** a **floating origin**, meaning everything is drawn relative to the camera, not the world's
middle, plus a standing pawn that holds exactly still. It touches every drawing part, which is why it
belongs with the renderer work (`vision/proposal-voxel-renderer.md`, "Camera-relative maths").

**Why it matters:** the world is big on purpose, and settlements will be built far from the middle.
Whichever Claude is moving the march's line inward, or touching how the near world is drawn: raise
it with David first.

**Co-op** (David asked, 2026-11-02; item 16): no conflict. The floating origin is each PC's own
drawing; every machine draws from its own camera. What the machines share is the world and the
simulation in true positions, so those must never be raw 32-bit numbers far out either: blocks as
whole numbers (they are), creatures and players as their chunk plus a small offset (or fixed point).
Then a far-off position means the same on every machine.

## 23. A pre-alpha for about 10 players, with a report file  [SOON; David wants it "the sooner ... the better"]

David, 2026-11-03: "the best test will be having a pre-alpha build fun enough that 10 people can play
and send us the report file back on their hardware and all its performance across several machines."
It replaces the weak-hardware test for now (`docs/decisions.md`, "The renderer: right and fast
together, erring on the side of right").

**Open, for David:**
- **What "fun enough" means:** the smallest game ten people would play for an evening. Not decided.
- **Who the ten are.**

**Counsel's notes on the report file** (nothing built; the logs already hold some of it):
- **What goes in:** the build number; graphics card, processor, memory, Windows version and the
  adapter actually used; window size; frame rate, 1% low, worst frame and card time by situation
  (standing, walking, flying); the crash and freeze reports; how long they played.
- **Privacy:** the player sends it themselves, and can read it first. No user names: Windows paths
  in logs carry them, so strip those. Say plainly what's in it.
- **Keep a way back:** until the reports are in, the old mesh path stays behind its switch, so a
  player whose machine can't run the march can still play and still report.
- **Things a player's copy must carry** (found 2026-11-04):
  - **The newer shader compiler (DXC):** build 59 uses the Windows SDK's on this laptop, and other machines
    fall back to the old one (FXC): about 50 s at every start with one core flat out and the march not
    drawing. Package it with the game (`docs/far-view.md`, "Build 59").
  - **David's render settings:** the code's defaults differ from his preset, and `presets/` isn't committed.

**The risk David accepted:** nothing yet tests the march on weak hardware, so what the ten find could
come late and cost more. The laptop's integrated graphics stays a no-code check for whenever he
wants it (`WGPU_POWER_PREF=low`).

---

## Settled

- **World size:** Grok's early figure (100,000-block radius) was replaced by `docs/decisions.md`
  (a circle ~72,000 blocks across). The docs are right.
- **Season length:** Grok discussed 21 or 30 days. The docs say 7 days per season, 30-minute days.
  The docs are right.
- **The 218-block list:** replaced by material × inclusion (see item 4). Grok also said 248; the list has 218.

# Incus Artus: the spec  [DRAFT for David to cut, 2026-11-10]

What this is: everything David has said the game must include or be, gathered from the docs, the journals and his own messages, sorted and de-duplicated, his to cut and correct. Nothing in it is new. (Refs are file:line; the key to the short names is at the very end.)
Status key: [decided] he decided it. [wanted] he wants it, with details or timing still to come (things he marked "later" included). [idea] his idea, not yet a decision. [open] a question he hasn't answered. [early] from the Minecraft-mod and Grok era, not touched since: does it still stand?

## In one page

- Incus Artus is "basically just minecraft and dwarf fortress at the same time": a big open-world immersive survival and settlement sim.
- Minecraft-style first-person building and exploring, with Dwarf Fortress-style living settlements; a commercial, standalone game.
- You can be a lone adventurer, grow from a campsite into a network of towns, or both, moving freely between first person and the management views.
- The name is Incus Artus, the anvil of the craft: Artus is craft, spelled with a u because he prefers it.
- The world is small but must feel big, and big but must feel small; it unfolds like origami as you discover it, its complexity shown only when needed.
- Nothing in it is truly random: a metaphysical world, the form, is projected into a physical one, "so that 1 always equals 1".
- It only has to pass a glance as real: fantasy and fun come before science.
- The world depends on NPCs acting of their own accord; they crave jobs, and the bustle comes from simple decisions.
- You can't fully control them; planning around their routines is part of the fun.
- Strict reasoning under the chaos gives an "orderly commotion" with meaning and reasons; like Minecraft, the world is seen by squinting.
- Light is precious: nights are very dark.
- Graphics beautiful, fast and read perfectly; the essence of the games he remembers, never the costume of "8-bit nostalgia"; every element owns its compromise as style.
- It must run "like a top": a Koenigsegg-style game, where tuning the engine matters as much as the content.
- The mouse feel is tight, never vague; moving the mouse should be a pleasure in itself.
- The game never becomes unwinnable, and only the player's joy is sacred.

## To settle

Where the sources disagree. Each version is given with its ref; nothing here is picked.

- **Day length.** Built and decided: a day of 15 minutes of light and 15 of dark (decisions.md:392; simulation.md:95). Wanted 2026-11-09: "30 minutes of sun and 30 minutes of night appeal to me", with sleep skipping the night, so a day is 30 to 60 minutes of play (vision.md:324-332, 900-902).
- **Seasons and the calendar.** Built and decided: a year of 4 seasons of 7 days (decisions.md:392; simulation.md:95). 2026-11-09: seasons counted in days, with no weeks or months; "7 day weeks then seem excessive"; "we may only need two seasons"; days per season open (vision.md:325-326, 357-358, 901-902). Older and still on file: spring and fall wheat on 2-week growing cycles, which a 7-day season can't hold (vision.md:834). The early 30-day months and 21-day seasons are in Appendix A.
- **The laptop's target.** 2026-11-05: on the RTX 5070 Ti laptop, mostly 250 uncapped; at a 120 cap never a stutter, no frame over 8.3 ms (decisions.md:984-989; regroup.md:27). The dyno's mark: a stable 237 (median 237, dips to about 180 fine, 120 the floor) (dyno.md:754-764). 2026-11-10: a smooth 240 (review:27). The resolution is settled: the laptop's own 2560 × 1600, not 2160p (David 2026-11-11, decisions.md:992-993). The heaviest moments may dip below 240 as long as 120 holds (decisions.md:1523-1528).
- **Lightweight graphics, or no expense spared.** "Lightweight graphics, so the computer's effort goes into making the world feel alive" (vision.md:19; decisions.md:907-908; conv:484, a Claude brief David pasted). Against it: "we're sparing no expense on the end product" (msgs:210); "this is a Koenigsegg style game" (decisions.md:1086-1088); a renderer good enough that everyone asks how (decisions.md:64-66; proposal:363). Both agree the processor is kept for the simulation (decisions.md:1569-1593).
- **How tall the mountains are.** Peaks at least about 400 blocks (proposal-peaks.md:9; WG:2452-2453). Peaks towering to about 512 (terrain-pipeline.md:194; proposal-peaks.md:25-26). "the mountains are eight hundred blocks tall, and thousands of blocks wide" (WG:1393-1394; watch-list.md:389-391). Peaks about 900 (placer.md:522). The ground is held under Z 333 until peaks need more (decisions.md:207).
- **World types and settings at the start.** Decided 2026-10-26: starting a game is just starting a game, with no settings and no world types, every world-gen parameter fixed at release (vision.md:642-645). Against it: world types offered to players as Tectonic Sim presets, decided (WG:325-327, 436; msgs:84); a new game picking the best of 10 worlds, idea (WG:487-488); empty-world game modes later, idea (vision.md:641).
- **Where you start.** Decided: in year 1000, "somewhere random" (WG:2151). Wanted 2026-10-30: on a road, at the spot farthest from any settlement (decisions.md:731-734; WG:489). Idea: you wake in your tent on a road (vision.md:50-51, 466-468).
- **Who the player's mother is.** Idea: the player is the God of Fire's son by a mortal woman (vision.md:50-51). Against it, a lore beat that was Counsel's idea, which David praised on 2026-11-09 ("your idea of learning that youre the son of fire ... is choice"): as you go to summon Darkness you learn you are his son of fire; he tries to kill you first, then decides corrupting you is the crueller blow to his unfaithful wife (mem/plain-game-text:18). That reads as if your mother were Fate, Darkness's wife. Praised as writing; not decided as lore.
- **Fate: god or goddess.** Early, David: "worshipping him" (conv:49). Later docs: the Goddess of Fate, married to Darkness (vision.md:40-43, 144); David: "theres two goddesses and one god that are not inimical" (decisions.md:398-400).
- **Rivers: does the water follow the land, or the land make way?** Recorded as decided: the land makes way (mem/rivers-walkthrough:13). Recorded as open: water follows the land, the land makes way, the webbing, or a mix: "im not ready to choose a solution." (WS:561-563, 809-810; water-inventory.md:203). Also wanted: rivers that roll like a marble between hills, deepening the natural valleys (WS:723-725).
- **River names and the great river.** Decided: ranks spring, creek, stream, river and great river (great 59 to 150 wide) (rivers.md:805; terrain-pipeline.md:62-63). Also decided: "great rivers aren't really a 'thing'", so the names stop at river (WS:84-86); "creek" is only colloquial, springs 1 to 2, streams 3 to 8 (WS:615-617). Agreed either way: size is area and names are labels only. A great river crossing a desert like the Nile is fine (WS:669).
- **How many ponds.** Wanted 2026-10-27: lowlands full of ponds (terrain-controls.md:85-87). Later: lakes and ponds chosen on purpose, "i really dont want many but theyre kind of obligatory" (WS:63-64); swamps pocked with many small ponds, placed deliberately (WG:1375-1376).
- **Do sub-block pieces ever occur in nature?** Open, remind David (decisions.md:244). Decided: ground, rock, caves and water are whole blocks, sub-block pieces come only from blueprints and players (quartile.md:486). Early: dirt and grass slabs exist but never occur naturally (conv:390). Decided: real snow falls as layers of 1 to 8 eighths, with drifts (WG:479-480).
- **Can bars overlap?** Decided: quarter bars may overlap (decisions.md:224-228; conv:378). Later set aside as an open question (quartile.md:526).
- **Shadows.** Decided 2026-11-06: sharp shadows matter only up close (decisions.md:1452-1461); close shadows exact enough to tell an enemy is round a corner (proposal:853). Decided 2026-11-10: blocky light everywhere with rough shadows; for actors, leaning to rough shadows everywhere, to be judged when things move (dyno.md:639, 889).
- **Water's look.** Open: transparent or not, perhaps opaque flowing water and transparent still water (decisions.md:463-469). Decided later: water is a see-through medium, blue, about 16 blocks deep, with no reflections (regroup.md:1252; mem/resume-2026-10-26-lift:35).
- **How hard making a world works the processor.** Decided: "its totally fine to roast the crap out of peoples cpu" when a world is made (decisions.md:126-128). Decided: world making uses half the cores by default, with a slider (rivers.md:200).
- **The test world.** Decided: "the test world should be retained", with fixed viewpoints (forest, village, water, glass wall, stairs, caves) (msgs:210; far-view.md:532; WG:1787-1788). Idea 2026-10-29: the test world retires, its features moving to a dev lab and a civilization and behaviour lab (decisions.md:652-657).

## 1 Identity and feel

### What it is
- Minecraft and Dwarf Fortress in one game: an open-world immersive survival and settlement sim, Minecraft-style first-person building and exploring with Dwarf Fortress-style living settlements: "basically just minecraft and dwarf fortress at the same time". [decided] (vision.md:12-14; conv:10, 38)
- A commercial, standalone game; the old Minecraft mod was only a step toward it. [decided] (vision.md:18; conv:16, 38)
- Play as a lone adventurer, grow from a campsite into a network of towns, or both. [decided] (vision.md:14-15; conv:16, 22)
- Called Incus Artus, the anvil of the craft: "artus is craft, not arthur ... i just prefer the u there." Arthurian legend is one of the lore's sources, not the name's meaning. "Incus Artis" is out of date. [decided] (decisions.md:496-503)
- Keep familiar conventions, for everyone. [decided] (simulation.md:329)
- Minecraft rebuilt from scratch with what hardcore mods, RLCraft, Vintage Story and Songs of Syx taught: more depth and quality of life, keeping Minecraft's joy. [idea] (vision.md:23-26; conv:529)
- Natural, logical solutions: no redstone; villagers given orders and farmhands do what a contraption would; big hauls by labour, wagons and donkeys: "villagers being given orders is better than redstone by far". [idea] (vision.md:27-29; conv:22, 529)
- Don't drift too far from Minecraft (a jeweller's lapping disk would be too far). [idea] (vision.md:837-838)
- After the intro "it's just Minecraft": accessible at once, a little more involved (like RLCraft or Vintage Story). [idea] (vision.md:729-733)
- Finding a village is where it goes far beyond Minecraft; at first it feels like Skyrim or Daggerfall. [idea] (vision.md:734-735)
- Content is generic myth and fantasy, medieval, with procedural stories and quests. [early] (conv:16, 470-472)

### Feel and tone
- Only the player's joy is sacred: "nothing that isnt ultimately part of the players joy is sacred to me". [decided] (decisions.md:948-949)
- A "Koenigsegg style game": tuning the engine matters as much as the content. [decided] (decisions.md:1086-1088)
- Tight mouse feel, never vague: "theres nothing worse than a vague mousefeel". [decided] (decisions.md:1353-1359)
- So responsive that moving the mouse is a pleasure in itself: "it becomes pleasurable just to move the mouse". [decided] (decisions.md:1455-1461)
- Make magic happen: "good enough won't make magic happens and we need magic". [wanted] (projection.md:20)
- Don't assume a difficulty setting; it's possible only once the game is basically complete: "it is implemented so badly on nearly like, all games". [wanted] (vision.md:778-780)
- Everything you can touch is a toy; every task fun and not too hard; instant gratification with depth underneath: "everytging you can touch is a toy". [idea] (vision.md:729-733; conv:182)
- It never pushes you too hard: you can ignore the world and live a Stardew Valley farmstead life, until the horde arrives, maybe. [idea] (vision.md:16-18)
- Threats must be taken seriously and reward taking part, not only hermit play. [idea] (vision.md:539-540)
- Horror comes from how enemies behave, not from the art style. [idea] (vision.md:12-13, 533)
- Game of Thrones-style politics and slowly building dread. [idea] (vision.md:84)
- Idyllic when nothing is wrong; turning to horror fast when something bad happens: "relatively idyllic when nothing bad is happening". [early] (conv:470-472)
- Also called a horror game: an "open world immersive survival and settlement simulation horror game". [early] (conv:438, 513)
- Content normative and universally inoffensive. [early] (conv:169)

### The world's life
- Strict reasoning under the chaos gives an emergent "orderly commotion" with meaning and reasons: "its the strict reasoning that enables the chaos". [decided] (decisions.md:16-20; projection.md:13-14)
- Like Minecraft, the world is seen by squinting, through imagination; the hustle and bustle hides the seams in the simulation. [decided] (jobs.md:64-67)
- The world is folded like origami and unfolds as it's discovered; complexity is shown only when needed. [decided] (simulation.md:7)
- Perceived complexity past what the player can take in, so it feels like life. [wanted] (proposal:853)
- Truly unpredictable, but held in bounds "like a tokamak". [idea] (simulation.md:178-183)
- Luck and coincidence are half of life. [idea] (simulation.md:213)

### The world's scale, felt
- Small but must feel big, and big but must feel small: "a big world that needs to be small". [decided] (WG:2458-2459)
- Scale carried by forests taller than Minecraft's, long roads, towns seen in the distance, and dangerous areas that take a long time to cross. [wanted] (WG:2456-2458)
- Big enough to go off into the wilderness, make a living, and be alone. [idea] (vision.md:120, 641)

### Who it's for
- Activities: exploring, gathering, searching, fighting, building, chores, organising; later managing followers and inhabitants, laying out regions and workshops, and the main questline. [idea] (vision.md:615-617)
- Players short-term, medium-term ("just wants to beat the game") and long-term (live in it); always two clearly forking paths: go looking for trouble or exploring, or build toward a slower, more cerebral power fantasy that keeps raising threats and needs. [early] (conv:182; from-grok.md:100-102)
- Instant appeal, with Dwarf Fortress veterans as the deep audience: "the appeal has to be instant gratification at hand at all times". [early] (conv:182)
- Everyone gets their own survival adventure with their own solutions. [early] (conv:529)

### The story's place
- The game never becomes unwinnable; endings are end states of the story, not the end of the game. [decided] (vision.md:70-72)
- The world lives a normal lifetime of procedural events, then drifts into an endgame as evil grows. [idea] (vision.md:15-16)
- The story gives context and a way to take control of the world; it never blocks doing Minecraft things. [idea] (vision.md:739-743)
- The story must be completable; finishing it leaves a world you can be rich in and build anything in. [idea] (vision.md:744-745)

## 2 The world

### Character of the land
- Fantasy over geology; pseudoscience that passes a glance: "people glance at it and think 'yeah that could be real'". [decided] (WS:648-650; WG:503-506)
- Water follows pseudoscientific principles tuned to look good in a game, not real physics. [decided] (WG:699-700)
- Dwarf Fortress-style geography at the broad scale, Minecraft's feel and proportions up close. [decided] (vision.md:622; conv:16, 442)
- Noise only as a final smoothing layer between deliberate transformations; no regular, repeating bumps or patterns the eye can pick out: "your average perlin generated terrain is kind of lame". [decided] (terrain-controls.md:81-84; WG:1832-1841; msgs:243)
- Worlds look natural and memorable, with enough ridges and plains in a pleasant arrangement. [wanted] (WG:1818-1820)
- Grand vistas: from a peak you look out over the land, not over an endless plain with a cliff nearby. [wanted] (WG:2452-2454)
- Ridges break a continent into "paths and rooms" big enough for a couple of towns, without strangling it; height and landmarks seen from the plains. [wanted] (handoff.md:280-281)
- Small-scale "regular irregularity" everywhere. [early] (conv:93)
- Domain warp as a tool for world-map features: "ok, i love the warp". [early] (msgs:414)
- Derive the forces behind the terrain backwards from results that already look right (wind from mountains, for example). [early] (conv:563-565)

### Scale and shape
- 1 block = 1 m (Minecraft scale). [decided] (decisions.md:201)
- A circle about 72,000 blocks across (radius about 36,000), about 70% water (tunable), continents inside; quality over size: "72,000 blocks is huge enough for anyone". [decided] (decisions.md:208; vision.md:621; msgs:213)
- Scale reference: a realistic island 30 to 40 miles wide. [decided] (WG:1525)
- One to three continents with ocean between them. [decided] (WG:283-286; msgs:51)
- The world is the circle inside the square field; an ice wall lies past it; few players will sail that far. [decided] (WG:201-202)
- The ocean empties quickly as you sail outward; a few islands near the continents. [decided] (WG:248-250; msgs:12, 54)
- Height runs from Z 0 (bedrock) to Z 1,023, raised so mountains tower over the land; the terrain's cut-off is its own setting. [decided] (decisions.md:206; WG:1571-1572, 2434)
- Sea level at Z 128. [decided] (decisions.md:207; WG:343; msgs:107)
- Vertical layout: bedrock at Z 0, a thin lava lake with stone podes holding rare minerals, dark caves, the surface, the sky. [decided] (vision.md:623-624; conv:59)
- The floor stays at Z 0 for now, with a way kept open to go deeper: "i would like to be able to go below zero if necessary". [decided] (WG:1392-1396; watch-list.md:387-392)
- Chunks are only a storage detail: no terrain or biome logic follows chunk lines; biomes are continuous fields and shift gradually. [decided] (decisions.md:213-215; vision.md:622; WG:2049-2050; conv:432)
- Regions: a Voronoi pass breaks the land into sized regions. [decided] (WG:2109-2110)
- Going deep is harder and more dangerous than in Minecraft. [wanted] (WG:1393)
- (Mountain heights: see To settle.)

### Making a world
- A new world is made completely before you enter it, every spring and creek included, and saved; nothing about the ground's shape is worked out at runtime; rivers and lakes are finished before you spawn. [decided] (decisions.md:954-957; WG:581-582; dyno.md:136)
- One world seed; every step takes its own seed from it, so one press remakes the whole world: "New seeds for everything". [decided] (WG:205-207)
- Same seed and settings, same world, in any order; the watching speed never changes the result; the planet's dice are never bent: "Same seed and settings, same mountains." [decided] (WG:255-256; simulation.md:104)
- Every world-gen parameter tuned by David; worlds big and not overpopulated. [decided] (vision.md:642-645)
- World creation takes about 10 minutes on a slightly below-average PC (980 Ti era, 4 to 6 cores) and a couple of minutes on the laptop, behind a loading screen that draws the generation live with lore and art. [decided] (decisions.md:955-956; WG:678-680; vision.md:699)
- The wait makes a world feel like a committed artifact and discourages rerolling ten worlds. [decided] (WG:680-681)
- The test of the world's maths is David's eye on his default presets, not the bits: "if i cant look at ten worlds and tell you which one...". [decided] (decisions.md:187-192)
- 1,000 years of history are run at creation with the same rules and Fate engine; the calendar is 0 at generation and play starts in year 1000. [decided] (decisions.md:415-417; WG:2148-2151; simulation.md:98)
- Good worlds should come routinely (aiming from about 90% toward 95%). [wanted] (WG:485-486, 1821-1824)
- A world rejector: make worlds and throw out those that miss criteria, with a quick tectonics rejector before baking; perhaps under a minute a world. [idea] (WG:485-488, 2388-2390)
- (World types and settings for players: see To settle.)

### The pipeline
- World generation in steps like Dwarf Fortress, replacing Minecraft's completely. [decided] (conv:42, 53; vision.md:640)
- The order: Tectonic Sim, then the World Compiler (landform, climate and rivers, soil and ores), then Ecology, Civilization, and the Interpreter. [decided] (WG:559-574; where-we-are.md:47, 66-68; msgs:186-202)
- The compiler decides and the interpreter draws; the interpreter never simulates water. [decided] (WG:572-580)
- The World Compiler covers everything up to non-sentient life. [decided] (WG:50-52, 588-590)
- Ecology reads the ground and never changes it, and comes after the whole world engine. [decided] (WG:570, 1589-1590)
- Mineralogy adds or removes no blocks; it only decides what they are made of. [decided] (WG:1589)
- Content is added horizontally through the pipeline (swampiness shaping ecology and topography; civilization placing structures and bad guys); the first answer is more stages further right, with a second "re-read" pass kept on file: "we can add content horizontally through the pipeline". [decided] (decisions.md:84-88, 97-119)

### Tectonics
- Plates are rags on a table, pushed around: wrinkled at the edges, with patchy stiffness; heavy bodies that crack or bend rather than spin: "plates are rags on a table being pushed around". [decided] (WG:235-237, 268-272; msgs:27)
- Mountains are where rock got squeezed; nobody places them. [decided] (WG:239-241)
- Ocean plates squeezed above the water become islands; ocean splitting leaves a V-shaped valley. [decided] (WG:248-250, 291-292; msgs:12)
- Plates get their motion at the start and coast to a stop; sliders find the "tasty ranges". [decided] (WG:251-252)
- Plates never reach the edge: a bowl keeps them near the middle, "like a beyblade arena". [decided] (WG:253-254; msgs:12)
- Each plate tilts a little and sits partly submerged, with a blend toward terrain noise. [decided] (WG:293-295; msgs:51, 78)
- Each plate gets its own spread of values so plates vary; land and ocean plates have separate settings. [decided] (WG:302-320)
- Plates weld where pressed together: some lakes and straits are fine, but no sea canyons through every range. [decided] (WG:1826-1829; msgs:113)
- No holes in the deep-ocean crust. [decided] (WG:1842-1848)
- The squeeze field is kept, for later shaping or as a mask of mountainousness, and its direction is shown on the map. [decided] (WG:295-296, 335-337)
- Subduction, rifts and new ocean floor as a later step. [wanted] (WG:321)
- Eventually the Tectonic Sim runs a world preset and hands its maps straight to the compiler. [wanted] (WG:1829-1831; msgs:87)
- Ocean plates give the continents' outlines their character, with enough squeeze to raise ridges without balling up: "i need the ocean plates to give the character to the continents outline". [wanted] (notes:5)
- Plates built from a finer grid of cells that cluster into organic blobs; ocean plates smaller. [idea] (WG:1850-1853; msgs:18)

### Coasts and lowlands
- Torn, spotty coasts in places (ponds, inlets, tide pools, islets, shoals); the torn coast stays. [decided] (WG:399-406, 2421; terrain-controls.md:85-87)
- Edge rise: a gentle, continuous, noticeable hill following the plates' edges. [decided] (WG:417-422)
- Coasts vary with circumstance: cliffs in places, simple coasts like Africa's west coast elsewhere, then shoals, shallows and islets. [wanted] (WG:1807-1810)
- A gentler change from mountains to plains; plate tilt grades the land, giving smooth coasts on one side and steep ones on the other. [wanted] (terrain-controls.md:39-42; msgs:119)
- An erosion pass to wear down rough edges; real water erosion later, as another additive. [wanted] (WG:368, 1561; msgs:137; proposal-peaks.md:136-137)
- The flats just under sea level: keep them as tidal flats, or remove them. [open] (WG:474-476)

### Mountains
- Real mountains you can overlook the land from: towering, fantasy-like, craggy, varied, never obviously repeated: "we hit it on the head". [decided] (proposal-peaks.md:9, 41-44; WG:2425-2426)
- Mountains with peaks, not plateaus. [decided] (WG:355-358; terrain-controls.md:55-56; msgs:113)
- Ridgelines are the hill country, a nest for the mountains; peaks are the mountains; both come from the squeeze; ridges cut into the peaks' feet, and steep towering peaks fade smoothly into the ridges. [decided] (terrain-pipeline.md:192-193; proposal-peaks.md:22-24, 106; WG:1523-1537, 1625)
- The "hope" look: small, sharp, short ridges broken into peaks every few thousand blocks, only on the most crushed rock; perfectly scalloped, irregular peaks, never a giant fingerprint: "breathtaking results... we've stuck gold." [decided] (WG:536-552)
- Peaks with sharp ridges and smooth valleys, as if eroded; irregularly fatter and cragged, so smaller mountains form round them: "crag them up real good". [decided] (WG:1541-1545, 1652-1654)
- Summit crags: one more jagged pass above a set height, with fine sculpting from the tip down to give the king peaks real jaggedness. [decided] (WG:1701-1706; notes:2)
- Overlapping peaks add up into massifs ("higher wins" kept as a switch). [decided] (terrain-pipeline.md:163; WG:2425-2426)
- No per-mountain settings: every difference between peaks is a rule for every mountain. [decided] (proposal-peaks.md:76-77; WG:1556-1558)
- Mountains need no direction; the ridges give the continental grain; perhaps a slight stretch later. [decided] (proposal-peaks.md:134-135; WG:1559-1560)
- Pits at the ends of fake-erosion valleys are fine. [decided] (proposal-peaks.md:136-137)
- Fantasy mountains, taller than they are wide, mostly in strings, sometimes in bundles with no ridge direction: "generally in strings. in some cases they make bundles". [wanted] (terrain-controls.md:106-108, 121-122; WG:1516-1517; conv:561)
- The climb: scalloped ramps from the lowlands up to the peaks, so a giant mountain shows over the ground in front. [wanted] (WG:2385-2386, 425-427)
- Long sloping arms as smaller peaks, with a narrow double-dip curve. [idea] (proposal-peaks.md:95)
- Second-layer mountains from scatterers, and fractal external rays in each range's own coordinates: "only look wrong to a geologist, not a Skyrim fan". [idea] (WG:503-520)
- Dots seeded along a range, carried on its heading, scattered and thinning; a range's tip like a carrot's thin end. [idea] (WG:1604-1616)
- Grain-drift and ridge-stream dots: useful tools without a use yet. [open] (WG:2423-2424)

### Hills
- Ground detail became the Hills: "its basically what i wanted from hills". [decided] (WG:1698-1700; terrain-pipeline.md:179-181)
- Foothills mostly forested; hills feathered softly from the mountains and the coast into the plains (the old Foothills feature was cut, Appendix A). [wanted] (notes:2)
- A hill field from the peaks' own idea, with less noise, growing out toward the plains so they are less flat; forests carry much of this. [idea] (WG:1581-1588)

### Climate
- A Climate stage after the landform, holding rivers, temperature, wind and rain, fine erosion, and springs through to deltas; its fields feed Ecology (ranges, habitats, plants, trees). [decided] (WG:698-704; terrain-pipeline.md:42-46)
- Rivers come first in Climate; climate may read the rivers, never the reverse: "rivers CAN mean something to them". [decided] (WG:1241-1245; WS:897-899; rivers.md:67)
- Climate never reads the carved ground. [decided] (WG:955-957; WS:754)
- Keep everything before flowing water as simple as possible: "Flowing water is the hurdle." [decided] (WG:1036-1038)
- Ice caps north and south and a hot equator band, drawn as squiggled lines; the two ice lines are equal. [decided] (WG:1098, 1216-1218, 1728-1729; vision.md:686)
- Milder by the sea, colder higher up. [decided] (WG:1220-1221)
- Temperature flows naturally, like strings between the three squiggles: it "isn't as simple as 'warmer in the center, colder north'". [decided] (WG:1715-1718)
- Tune the ice caps so every world gets a reasonable amount of ice; the equator band gives one or two dry spots. [wanted] (WG:1217-1219)
- Later: chemistry on blocks and regions (soil), irregular winters that moderate plant growth, and temperature as an emergent environment like the "hell of sand" games, with Noita as the aspiration. [wanted] (decisions.md:348-349; block-properties.md:28; vision.md:638-639)
- The lake effect (lakes adding rain downwind): left out to see if climate works without it, possible now that rivers come first. [open] (WG:1164, 1244; WS:908-909)

### Rivers: the rules
- Rivers don't come from rain; wetness for Ecology is rain plus nearness to water: "We shouldn't confuse rivers with rain." [decided] (WS:733-735, 757; rivers.md:712)
- Rivers only need to look right. [decided] (rivers.md:714)
- Water always runs downhill and always connects. [decided] (WG:586-587)
- Water joins by a face, never only at a corner; the line's rounding stays: "keep the rounding, add the anti-kitty-cornerer." [decided] (WS:222-237; rivers.md:214)
- Least change, with thresholds for filling and cutting, and the intelligence put into the rivers; a filled hollow stays between its banks. [decided] (WS:222-225; rivers.md:255, 271)
- The river network grown from the mouths, choosing branches of the drainage tree; its Dwarf Fortress-like placement kept: "decent and very Dwarf Fortress-like". [decided] (WG:819-823; rivers.md:745; WS:720-722)
- The backwards method: deltas at the lowest, flattest points, not too close together; the climb favours long flat runs, then steps; springs that reach no river find a lake or pond or fizzle; lakes overflow; the obvious variables as sliders. [decided] (WG:787-805; WS:742-745)
- His cross-section drawing wins; profiles wider. [decided] (rivers.md:719)
- Rivers are done enough to go on; more work only in a bigger rework with a specific goal: "i'd say we've pretty much got rivers fixed". [decided] (WS:6-11; terrain-pipeline.md:105; mem/rivers-walkthrough)
- David will write many river-type rules later. [wanted] (rivers.md:723)
- (Whether water follows the land or the land makes way, and the river names: see To settle.)

### Rivers: the network and its look
- Gorges up high; down low, rivers wind round the terrain: "they're better up high." [decided] (WS:74-76)
- A big gorge with a lake way up high is fine. [decided] (WS:6-8)
- Islands and braids kept, and repeated if possible: "must be preserved, as it is great, and duplicated if possible". [decided] (WS:730-731; rivers.md:783; terrain-pipeline.md:97-99)
- Waterfalls and rapids of all shapes and sizes are a big wow factor: both rapids of many small falls and bigger falls. [decided] (WG:782-783; WS:305-317)
- Waterfall curtains: water blocks down a fall's face, dropping at the lip; no water standing against air; no raised square pools or raised sandbanks in the sea. [decided] (WG:928-930; rivers.md:316; WS:740)
- Rivers running side by side merge with a waterfall, the higher falling into the lower. [decided] (WS:305-313; rivers.md:225)
- A tributary's end eases down linearly toward the river it joins. [decided] (WS:479-480; rivers.md:236)
- More joins are no worry unless too few rivers are left; the fix for too many whole rivers joining is shelved until the problem is seen. [decided] (WS:100-101, 329-331)
- Deltas are triangles about 64 to 128 blocks wide at the mouth, with 2 to 4 channels. [decided] (WG:772-773, 940-1012)
- A great river crossing a desert, like the Nile, is fine. [decided] (WS:669, 734-735)
- Petering out only where the water has nothing else to do. [decided] (WS:62, 555)
- Rivers spread round the world, fanning out through the terrain: "Not too evenly though." [wanted] (WG:787-789; WS:720-722)
- A pleasant, bridgeable web of springs, creeks and rivers: "a *pleasant* webbing of little springs and creeks and rivers that have bridges". [wanted] (WS:717-719; rivers.md:752)
- Rivers follow the land, curving between hills, finding the natural valleys and deepening them, running through the middle between two mountains. [wanted] (WS:723-725, 610-611; rivers.md:768)
- Long rivers from the highlands to the flats; valleys, not ponds, in rough ground; waterfalls where the plan steps. [wanted] (water-inventory.md:270)
- Big areas of land have few waterways, collected into one river. [wanted] (WS:609)
- On steep ground springs run deeper than wide, with falls and drops; springs high up have lots of waterfalls and can be very shallow, "just a 1 block path of water trickling down the mountain". [wanted] (rivers.md:138; WS:599-600)
- From above, a waterfall reads lighter (white water) than the river around it. [wanted] (WS:732; rivers.md:765)
- Two whole rivers joining should be rare. [wanted] (WS:84-86)
- Water you can find: visual cues on plains (tall grass, lushness) and sound cues (frogs). [wanted] (WG:1206-1208)
- Waterways drawn toward nearby water: a river that cuts anyway heads for the nearest pond or lake, the pull acting only at its tip. [idea] (WS:319-335)
- Islands placed on purpose, later. [idea] (rivers.md:793)
- Dry beds: use the temperature layer to take water out of some beds. [idea] (WS:662)
- The webbing: block-scale contours forming a soft trace of every passage between rises, for water to follow. [idea] (WS:869-872)
- Petering out may not be needed at all if the rest works. [open] (WS:676-677)

### Rivers: sizes, springs and banks
- Size is area, one continuous number; names are thresholds and labels only: "treating them as discrete types of water is misguided". [decided] (WS:44-47, 421-426, 641-654; rivers.md:143)
- Width against depth comes from the slope: steep is deep and narrow, flat is broad and shallow, from spring to delta. [decided] (WS:49-51, 81-82, 620-626)
- Water grows unrealistically fast on purpose, slowly by running and quickly by joining, a join adding most of both areas; joining is what makes a stream a river: "has to be unrealistic to get satisfying proportions". [decided] (WS:48, 644-651)
- Sizes by area are an experiment behind a checkbox. [decided] (WS:61, 419-425; rivers.md:122)
- Springs are 1 block and come from the mountains and the hills; the existing spring methods (plain or relative height, spacing and jitter) stay. [decided] (WS:44-45, 56-58, 595)
- Shores flush by default, level with the water so you can walk out, with one-block ledges only by context; lake and pond shores not always flush. [decided] (WS:70-73; rivers.md:189)
- Banks fade several ways; a cut bank and point bar; the bed steps a block at a time; bigger rivers wider and shallower. [decided] (rivers.md:721, 841)
- Where the land makes way: well feathered, gentle, as little change as possible; crags beside water are lowered: "the less it changes the land, and the more gradually, the better". [decided] (WS:83, 814-822)
- How deep a cut gets before a river looks for a way round is a slider. [decided] (WS:74)
- Hill springs come out gently, wander across meadows and peter out more often than steep ones. [wanted] (WS:596-598)
- More small springs in the flatter areas, or big rivers cutting through them from the highlands. [wanted] (WS:728-729)
- Banks slope toward the river at least one width; no notched streams. [wanted] (rivers.md:771-773)
- Later: ledges in hard rock, terraces along big rivers. [wanted] (rivers.md:915)

### Lakes, ponds and swamps
- Lakes chosen by a count range, as the peaks are. [decided] (WS:77-78)
- Lakes aren't raised square pools; lake rims are filled up to the surface rather than lowered. [decided] (rivers.md:758; WG:928-929)
- Lakes can feed rivers; ponds only take them in. [decided] (WG:775, 854-855)
- Tarns and marshes: "later or never" (marshes since replaced by the swamps below). [decided] (WG:1099-1100)
- Some lakes and ponds above sea level. [wanted] (WS:738; rivers.md:764)
- Far fewer lakes and ponds in rugged regions; only a few in the mountains, spilling into a stream on the way to the sea; not every crag pit a pond: "way way way less lakes and ponds in the rugged regions". [wanted] (WS:602-603, 739; water-inventory.md:224)
- Swamps pocked with many small ponds, placed deliberately: "we definitely need to place swamps". [wanted] (WG:1375-1376)
- Pick lakes and ponds before the water moves, to control how many there are. [idea] (WS:536-538)
- (How many ponds: see To settle.)

### Water in play
- Water doesn't simulate flow: "flowing" means a direction and a speed (which way the palette cycle runs and what it carries), as in Dwarf Fortress. [decided] (decisions.md:444-446)
- Water is plain blocks placed in the carve for now; water behaviours wait until the graphics are done: "until the graphics are done we won't be messing with water behaviors". [decided] (WG:907-911; regroup.md:961; handoff.md:27-28)
- Water in eighths: a bucket holds one full block; spilled water spreads in 1/8 layers like Minecraft snow, can be mopped up, and evaporates 1/8 at a time: "a 1x2 block hole would result in two blocks with 4/8". [decided] (decisions.md:447-462)
- Natural water has no levels, except perhaps rapids and falls (graded levels, speed, mist or splash foam). [decided] (decisions.md:449-450)
- Ponds, lakes and the sea past a river mouth are still; only edge blocks get the shore-ripple palette cycle, drawn from David's ripple tile map. [decided] (decisions.md:451-455)
- Water wheels: diverting a river may be required, so dug channels fill from rivers with a direction and a speed. [wanted] (decisions.md:470-471)
- Does diverting a river take water from downstream? Does evaporation empty pools, and does rain fill them? [open] (decisions.md:484-486)
- A fictional geology that explains springs to creeks to rivers to the sea, and how caves formed. [early] (conv:61)

### Shores and surface materials
- Shores and beach sand placed by distance from water, not by a Z level. [decided] (WG:478; rivers.md:555)
- Surface materials by David's numbers: snow when cold, gabbro above Z 333, stone above Z 256, beach and desert sand, dry grass, grass; the sea floor sand, then gravel; river beds gravel or sand. Gabbro marks the peaks: "so i can pick out peaks at a distance easier". [decided] (placer.md:551; terrain-controls.md:98-101)
- Today's snow should be snowy grass; real snow comes from rainfall as layers of 1 to 8 eighths, drifting with the wind; the speckled grass and snow border is feathered. [decided] (WG:479-483)
- Seasons and biomes shown as real blocks rather than palette shifts. [idea] (WG:170-171)
- (Sub-block pieces in nature: see To settle.)

### Caves
- Caves are formed deliberately, like rivers: planned, then carved from the recipe; no random caves; noise only dresses their content: "there are no 'random' caves". [decided] (WG:1268-1273; caves.md:17-18)
- Fun before science: "the important thing is they are fun." [decided] (WG:1250-1251)
- Cave biomes as kits by cause, with life underground. [decided] (caves.md:19, 22)
- The first cave: a simple dead end worming into a mountainside (the waterfall cave is deferred, Appendix A); rivers making fun caves is a happy accident, not a requirement. [decided] (caves.md:335-337; WG:1250-1251)
- Caves come as early in the order as works, with a panel of their own. [decided] (caves.md:338)
- Caves are pitch black unless lit; daylight reaches a little way in from a mouth. [decided] (decisions.md:393-397; caves.md:13, 456; handoff.md:196)
- A cave or hall big enough to be seen at a distance is seen at a distance, blending seamlessly with the near view: "it should be seen at a distance". [decided] (decisions.md:1476-1479; mem/counsel-resume:50)
- Cave mouths are lit and seen from a distance; lights inside show from outside, and a statue at a mouth shows if there's light on it; caves with braziers reading from outside at night is a key test: "lights have to be visible from inside the cave." [decided] (WG:1307-1315; caves.md:21; proposal:375)
- Cliffs and overhangs as cave features that broke out (overhang, mouth, rock shelter, arch), plus other carved features here and there, all visible from a distance. [idea] (WG:1340-1343; watch-list.md:385, 395-396)
- Badland canyons could be placed by the placer: "i think if i wanted i could even just place badland canyons." [idea] (WG:1373-1374)
- Exciting cave systems, geologically consistent and readable for adventure. [early] (conv:61; msgs:200)
- (Monsters and dwellers underground: section 9.)

### Rock, strata and ores
- Many stones, soils and ores; a block is material × inclusion, ores are overlays with percentage chances (galena gives lead, sometimes silver), and density and grade are fixed at "planet forming". [decided] (vision.md:625-626; decisions.md:257-258; placer.md:154; conv:10-12)
- The ores: bismuth, copper (native, malachite, tetrahedrite), gold, silver, tin, zinc, iron (hematite, limonite, magnetite), lead, nickel, platinum. [decided] (vision.md:627-629)
- The mineralogy that places ores also places the viable mine sites; a site stops being one once enough of it is dug out. [decided] (decisions.md:268-274)
- Every area diverse but viable (fuel, copper, tin, iron, flux, salt): "making sure that areas are diverse, but viable, will require careful balancing". [wanted] (ores-by-depth.md:3-4; placer.md:315)
- Some of the mineralogy centred in the mountains, as features of central interest. [wanted] (WG:1394-1395; watch-list.md:389-391; placer.md:529)
- Ores drawn from a Minecraft, Vintage Story and Dwarf Fortress cross-reference, ranked by depth over about 130 blocks of crust. [wanted] (placer.md:288)
- Prospecting can be a skill. [wanted] (decisions.md:269)
- Minerals are rock: an ore rock is building stone too, and drops ore, rocks and boulders, or a whole block if quarried: "hematite is a rock, you can make blocks with it". [idea] (decisions.md:306-308)
- Native ore inclusions overlaid as in Minecraft, perhaps marking where mines go. [idea] (decisions.md:309-310)
- Mineral tiers: common minerals with inclusions; less common ones (chromite) that drop ore; useful metals; economic or decorative metals; magical or alchemical ones. [idea] (decisions.md:311-314)
- The common minerals' colours come first, so mountains and stone buildings look good. [idea] (decisions.md:315-316)
- Three or four strata (top, average cave layers, deep, ultra-deep): the top two rich and convenient for every main need; the deeper and scarcer, the more exotic. [idea] (WG:1421-1425)
- In the ultra-deep, titanium and adamantine as cylinders through the magma lake at the bottom, with a risk of magma flooding. [idea] (WG:1421-1423)
- Layers follow the surface above them, with upsets; built from base bands, disruptions, inclusions, deposits and local distributions by biome. [idea] (WG:1426-1433)
- Balancing ore depletion between settlement miners and solo play: "it'll require careful balancing". [open] (decisions.md:279-282)

### Biomes
- Biome cores: small, distinctive, extreme regions plotted into the climate as peaks are, each strengthening the ecology there (spawn rates, beasts, resources, structures) and gating rare resources that draw you in; everywhere else is the blend. [decided] (WG:1454-1456, 1488-1492; terrain-pipeline.md:46-47)
- Eroded badlands are out: "probably too much to take on". [decided] (WG:1462)
- The God of Darkness has a layer of influence at the base of the world's activity, out of view; some areas are naturally bad-aligned: haunted, spooky or spider forests with phantoms, a haunted graveyard, huge spider nests. [decided] (WG:2120-2122)
- Weirdness as a field ("the gods have been here"), like rainfall and temperature: weird places produce a set of chance encounters (Ma'aiq the Liar, the orc seeking an honourable death). [decided] (jobs.md:52-54; vision.md:970-973)
- There must be some desert, for giant ants and scorpions. [wanted] (WG:1459-1461)
- Ice at the poles. [wanted] (vision.md:686)
- Fewer biomes than Minecraft, Dwarf Fortress in spirit but not as broad. [idea] (WG:1457)
- Deserts rare and placed with care; drylands are fine: "drylands, sure". [idea] (WG:1458-1459)
- A desert picker: one or two per map, pushing the drylands further as the peaks push the ridgelines. [idea] (WG:1463-1465)
- Darkness as an environmental layer; volcanoes that can appear in odd, weird places; volcano biomes to brave for the God of Fire's quest, possibly with dinosaurs; a structure in a lava lake "that goes down and down". [idea] (vision.md:968-973)
- Biomes from Dwarf Fortress with sub-biomes, shaped by a goodness-and-evilness environment layer, with necromancer towers in evil areas (perhaps now the darkness and weirdness layers). [early] (conv:51; from-grok.md:74-75)

### Plants and trees
- Trees: alder, ash, beech, birch, elder, elm, fir, hazel, larch, laurel, linden, maple, oak, pine, poplar, spruce, walnut, willow, yew, apple, orange, pear. [decided] (vision.md:631-633)
- Trees are made of whole and quartile blocks (quarter logs, thinner branches, in-between shapes), with a couple of growth stages that change at random; the first structures were oak trees. [decided] (decisions.md:576-581; quartile.md:14; WG:2300-2301)
- Plants are objects, 2D like grass or 3D like corn, and change over time. [decided] (decisions.md:674-681)
- Grass, flowers and small plants, with crossed-sprite grass. [wanted] (proposal:85; handoff.md:220-226)
- Far mountains with forests, later. [idea] (regroup.md:611)

### Animals
- Animals are part of the terrain: a likelihood per area, made with the land; individuals are real only in the Local simulation or as pets; no migrating herds on the books. [idea] (WG:705-708; vision.md:1002-1004)
- (Creatures, beasts and hostile places: section 9. Hunting: section 6.)

## 3 The look and sound

### The aim
- Graphics beautiful, fast, and read perfectly to the player: "beautiful to look at, fast, and reads perfectly to the player". [decided] (decisions.md:979-983; regroup.md:20)
- A beautiful world, squinted: an AAA-polished world seen at about 640×480 is the right look for distant terrain; high fidelity at low resolution. [decided] (decisions.md:1090-1094; regroup.md:424)
- The essence of the games he remembers (dark rooms, black screens, high contrast, every colour meaning something), never the costume of "8-bit nostalgia". [decided] (decisions.md:439-441)
- Every element owns its compromise artistically: shortcuts (low resolution, sprites, fog, dithering) are shown as chosen style, never hidden as defects: "every element has to own its compromise artistically". [decided] (decisions.md:520-525; mem/graphics-goal:19)
- Where something must be imperfect, it looks optical (dirty glass, grain, a lens), never digital: "almost like the eye sees it as dirty glass". [decided] (decisions.md:1186-1190; regroup.md:633)
- Graphics layered, each method used where it works best, so players never see behind the curtain. [decided] (WG:1869-1870)
- The "secret recipe": great performance and great looks together. [decided] (WG:2157-2158)
- Milestone: a DOS-game resolution where everything is there: "*but it's all there, dynamic, honest, and unflawed*". [decided] (decisions.md:1120-1122; regroup.md:139)
- No seams or flashes where near meets far; little things break immersion: "little things do matter". [decided] (far-view.md:265; WG:1770-1773; caves.md:850; msgs:237)
- Visible flaws cleaned before performance work; the ringing on flat ground and water must go; no pale rim where mountains meet the sky. [decided] (regroup.md:612, 892, 1307; decisions.md:1172-1174)
- A renderer you can forget about, handling anything new with no special case, good enough that everyone asks how: "freedom from asking how each new thing has to be handled". [wanted] (decisions.md:64-66; proposal:363)
- An LOD system better than anything out there: "an lod thing thats better than anything out there". [wanted] (watch-list.md:289-290)
- The sky feathered into skylines: thin skyline squares lean to the sky while a mountain's body is left alone, for next to nothing: "that actually works great". [decided] (decisions.md:1208-1214)
- Weighting kinds of pixels as a cheap polishing tool, once the speed is where it should be: "may be a very cheap and useful tool for a lot of things". [wanted] (decisions.md:1215-1218)
- Things must not vanish up close: "chests dissapear at 8 blocks away and that's annoying". [early] (conv:426)

### How the world is drawn
- One world projected the same at every scale, on processor and card: the far view is the same world, coarser. [decided] (review:72; decisions.md:39-43)
- The renderer treats everything the same: caves from the recipe, player-dug holes, mines, great halls and castles all reach it the same way, with no IDs: "i think its cleaner for the renderer to treat everything the same". [decided] (decisions.md:44-54; caves.md:193)
- The far view is ray-marched, in a dome of cells fixed to the world (no shimmer when turning): "its the hard part, we might as well do it now". [decided] (decisions.md:61-77; far-view.md:3; handoff.md:181-188)
- The world is marched and things are drawn: models, sprites, creatures, water and glass go over the march; a hybrid renderer is allowed; a somewhat exotic far view that pixelates naturally, and naturally coarse light, are still wanted. [decided] (decisions.md:78-83, 979-983; regroup.md:17)
- The near look may change: "im not attached to the near look". [decided] (decisions.md:78-83; far-view.md:496)
- Blurry up close is acceptable if it is equally and correctly blurry at every distance; the far view and its blend are the hard problem: "equally and *correctly* blurry at a distance". [decided] (decisions.md:1071-1081)
- Twin layers, not LOD rings: a low-resolution, high-fidelity layer and a high-resolution detail layer that always agree, like cels: "they're just aimed differently". [decided] (decisions.md:1102-1115; regroup.md:425)
- The switch between twins is a shaped blend curve in tunable stages, like multistage baja-truck shocks; layer seams feathered, with a slider strip per step. [decided] (decisions.md:1123-1125; regroup.md:464, 1235)
- Pixel sizes in bands of 1, 2, 4 and 8 by distance, each scale where it looks best; 8×8 looks good far off; he liked 2× finest and 4× coarsest; the coarsest is selectable: "8x8 actually looks good for the far distance, on this screen". [decided] (decisions.md:1169-1181; regroup.md:604, 635, 814, 824)
- No mismatch between scales; pixels shifting between them read like film grain: "seem much more like film grain". [decided] (decisions.md:1182-1185)
- Gliding coarse edges (PS2-era antialiasing, Half-Life on a CRT) kept as character: "i like it, its organic". [decided] (decisions.md:1191-1194)
- Sharpening in view is fine if very temporary, predictable, always in the same order, and eased rather than snapped. [decided] (decisions.md:1146-1152)
- Far detail shrinks with distance; no ziggurat staircases on far slopes; a smooth, adaptive far surface with lighting and shading from the recipe everywhere. [decided] (far-view.md:1229; regroup.md:1028; WG:2171-2172; watch-list.md:280-281, 319-322)
- Contour bands show the land's shape, the same at every distance; a coarse cell's top shows the average of what it covers. [decided] (far-view.md:928, 1159)
- Near blocks are hard-edged and lit by their facing, fading smoothly into cells: "this honestly looks amazing". [decided] (decisions.md:1198-1202; regroup.md:721)
- Near chunks lit to match the hill-shaded far terrain, not the other way round. [decided] (mem/graphics-goal:17)
- No rule is needed for what's seen inside solid rock: "don't worry about flying through the rock". [decided] (decisions.md:1203-1205; regroup.md:702)
- Distant terrain only has to look good (a Strife or Hexen backdrop is enough), but an important thing shows correctly from its first pixel: "as long as he can see something important from the moment its a pixel". [decided] (decisions.md:1523-1531; regroup.md:1084)
- An artistic upscaler for the distance, fast and a joy to look at, with styles layered; no upscaler may shift a block's colour into another's. [decided] (decisions.md:526-543; proposal:187; mem/graphics-goal:19)
- Temporal reuse used artfully: anything fast or unpredictable (animals, monsters, people, effects) never takes from history; pixels with no history get rays first; edits and light changes drop history; near block faces stay crisp. Reference: Playdead's INSIDE talk (GDC 2016): "things like taa i think can be applied artfully". [decided] (decisions.md:1210-1232; dyno.md:969; handoff.md:69-72)
- Distant terrain like Distant Horizons, out to the whole world; outflying chunk loading shows distant terrain, never a hole; distant structures stay visible in simplified form: "do it the right way". [decided] (WG:1767-1773; msgs:210-213; conv:442)
- Far away, trees become sprites of themselves. [decided] (decisions.md:584)
- Creeks narrower than the far terrain fade in by colour rather than popping in. [decided] (WG:1002-1006)
- Built things and towns hold up at a distance; towns, bridges and roads read from the sky and are lit at night. [wanted] (far-view.md:890, 895; marketing.md:30-34)
- Resolution falls off as a continuous curve warped by distance, with a slider for how fast the terrain pixelates and an optional snapped-grid overlay. [wanted] (proposal:311, 727; decisions.md:1516-1521)
- The softening blend at the near and far seam comes back with trees and rivers. [wanted] (watch-list.md:363-365)
- Crunchy 2×2 pixels may be a good look. [idea] (regroup.md:900; handoff.md:45)
- A mosaic snap filter; a world-anchored dome for far pixels, where a far dragon may lag. [idea] (proposal:327, 335)
- Pixel horizon: a crisp, low-resolution pixel-art far band lit from the true slope. [idea] (watch-list.md:299-304)
- Far people and animals as animated sprites; crowds as sprites past about 60 to 100 blocks. [idea] (watch-list.md:310; proposal:87)
- Caravans crossing thin forests seen from half a mile away. [idea] (proposal:857)
- Which things count as "important". [open] (decisions.md:1528-1529)
- Ambient occlusion and matching shading wait for a later graphics pass: "the barest thing to get me by until we do graphics more devotedly". [early] (mem/graphics-pass-deferred:11)

### Light
- Light is precious: nights are very dark, caves pitch black unless lit: "i want light to be precious". [decided] (decisions.md:393-397; WG:1304)
- Light per face, blocky and quantized, everywhere: "blocky and quantized lighting is good all around". [decided] (decisions.md:1403-1411; dyno.md:294, 639)
- Light only lightens and darkens David's chosen colours. [decided] (proposal:753; WG:1872-1874)
- No reflections and no bounce light. [decided] (decisions.md:1452-1461; regroup.md:1252)
- Moving torches and light from effects are required: "moving torches are obviously necessary as well as light from fx". [decided] (decisions.md:1452-1461)
- Placed light falls off in clear steps per block, 16 levels as in Minecraft (smooth only as an option): "very clear steps in brightness". [decided] (decisions.md:1462-1464; regroup.md:1246)
- Placed lights flood through air, coloured, stopped by walls, and flicker; per colour the brighter of two wins. [decided] (regroup.md:1092)
- Coloured, flickering placed lights (the brazier) sparkle from far away: "i can see them from far away sparkling." [decided] (objects.md:342; handoff.md:203-204)
- Far lights: their glow and twinkle shrink and fade into fog, never snapping off. [decided] (regroup.md:1292)
- Coloured light carries meaning. [decided] (caves.md:477)
- Terrain light from sun and sky may lag about a quarter second; light that moves with actors stays prompt every frame: "terrain light almost needs zero dynamism for gameplay". [decided] (decisions.md:1439-1446; dyno.md:408)
- One sun with direction and height, switchable sun shadows, and sky light that gives depth: "the sky light helps give depth to farther terrain". [decided] (handoff.md:86-96; regroup.md:763, 806)
- A warm-sun slider David blends to taste. [decided] (decisions.md:514)
- Day and night with dusk; by default night as dark as a moonlit night: "the dusk was beautiful ... looks really good". [decided] (regroup.md:1101; mem/resume-2026-10-26-lift:47-49)
- Carried light: a held lantern lighting about a 32-block cube round the player. [wanted] (regroup.md:1092; handoff.md:57-58)
- Light sources underground: a magic brazier from a cult lair, glowing lichen, lanterns. [wanted] (caves.md:456; handoff.md:213)
- Sun shafts, if they feel organic; the usual god rays are too heavy, so perhaps two rays inferred from the sun and the blocking shadow, drawn as bands of lighter shade: "sunrays may be cooler than i had previously thought, if they feel organic". [wanted] (decisions.md:516-519, 1454-1455)
- The sky light's tint and the sun and shade colours wait for David's word. [open] (regroup.md:851; handoff.md:94-95; WG:2174-2175)
- (Shadows: see To settle. Moonlight: section 11.)

### Sky, weather and fog
- Fog: a couple of miles of average visibility (about 3,000 blocks) for a deep sense of smallness; thickest at sea level and thinning with height, lying low in hazy valleys under clear peaks, the land fading to the horizon's colour and mountains melting into the sky: "fog looked sick." [decided] (decisions.md:505-507; regroup.md:919-920; mem/graphics-goal:15)
- Fog comes with the weather, not as a fixed wall: on clear days, from a high point, you see across the whole world, so the whole world must be drawable: "on clear days from a high point you should see across the entire world". [decided] (decisions.md:1160-1164; regroup.md:493)
- Skies: a couple of colours dithered and offset; sunsets in deep red, orange, magenta and blue, like an old adventure game's title screen. [decided] (decisions.md:508-509)
- Clouds: super simple, almost cartoony sprites. [decided] (decisions.md:510)
- Thunderstorms and rain are important to get right: lightning made in the FX Lab; thunderclouds as dark sprites that block the sun, fade toward dark and roil. [decided] (decisions.md:511-513)
- Particles in any colour: rain, snow, mist, foam, lightning, magic. [wanted] (proposal:91)
- A chosen weather event (mist) on the first morning, to cover the world still being made: "im not against a bit of stage magic". [idea] (vision.md:724-725)
- Wind ripples across grass in expanding rings downwind. [idea] (WG:158-161)
- Mountain colour contrast is David's to tune. [open] (regroup.md:934; decisions.md:1169-1174)

### Colour
- The world uses about 600 chosen colours, each carrying meaning (an echo of Boorman's Excalibur), freely lit and shaded, not snapped to a palette; effects may use every colour; the UI and HUD use 256 colours at lower resolution. [decided] (decisions.md:429-436; WG:1870-1875; watch-list.md:187-196; mem/graphics-goal:11-14)
- Colour by palette swap: textures use shade slots that each material fills; a colour session with David comes before the real textures. [decided] (decisions.md:255-256; watch-list.md:191-192)
- No programmatic dithering for now; perhaps later for sunsets, clouds or animated water. [decided] (WG:2081-2083; toy-factory.md:160)
- Palette cycles done deliberately: water direction, shore ripples, wind over grass. [decided] (WG:141-144; decisions.md:442-446; proposal:90)
- The master palette isn't chosen yet; textures are placeholders. [open] (decisions.md:487-488)
- Whether the finished frame (or only the far horizon) snaps to the world's ~600 colours, decided later by eye: "we just don't know". [open] (decisions.md:437-441; WG:1874-1875; watch-list.md:307-308; mem/graphics-goal:13)

### Textures and surfaces
- Block faces are 32×32 PNGs that tile seamlessly. [decided] (decisions.md:489, 549, 553)
- Random 90° rotation for faces with no "up" (grass tops), switchable per block; random face variants (some logs with knots) picked by position. [decided] (decisions.md:345-346, 554-556)
- Each soil has its own texture; the grass edge is one shared overlay. [decided] (decisions.md:558)
- Thin cut-out lines never vanish in the distance. [decided] (decisions.md:572-573)
- Textures crisp up close, mipmapped as they shrink, shrinking toward the far colour: "they look good." [decided] (regroup.md:936; handoff.md:27, 48-50)
- Joined bars are seamless, the texture running across the join; bars wear their own side and end textures (his bar art later). [decided] (quartile.md:72, 191; regroup.md:1024; handoff.md:31-35)
- Later: distant textures made small by Claude and touched up by David; random texture variants; animated textures. [wanted] (decisions.md:559-560)
- Overlays (ore, moss) as flat decoration, random sets (grass sprites, moss patterns), and alternative textures for one stone: "id like to be able to make different kinds of the same stone too". [wanted] (objects.md:392-396, 436)
- Glass far better than Minecraft's. [wanted] (proposal:121)
- Connected textures (a glass wall framed round its outside; shore ripples). [idea] (proposal:162)
- Contrasting textures darken as they shrink (far colours average stored values). [open] (handoff.md:51-52; regroup.md:954)
- Water caustics. [early] (conv:436)
- (Water's look: see To settle.)

### Figures, objects and effects
- Objects are models (the brazier, with an animated flame sprite): "it looks really good... they look sick though". [decided] (objects.md:3-7, 337; regroup.md:1189)
- Food's look matters for coziness: "so wording and appearance matters". [wanted] (vision.md:398-399)
- Figures: Minecraft-style figurines, slightly more detailed, with elbows (and presumably knees): "vintage story invented elbows ... so im leaving them elbows". [idea] (vision.md:944-945; conv:113, 119)
- Quake-flavoured modelling: "i liked how quake looked, quick and crispy". [idea] (vision.md:946-947)
- No gore beyond blood particles: a carcass pops apart. [idea] (vision.md:382-390)
- Spells and effects throw or cause bouncing sprite particles. [idea] (decisions.md:689-692)
- Dwarves in an 80s-metal and D&D poster art style. [early] (conv:127)
- Culture looks as golden ages: humans Roman, elves ancient Greek, dwarves Hittite, Anatolian and Urartian, "overbearing brutalist squared off dwarven aesthetics". [early] (conv:346-350; from-grok.md:68-69)
- People wear ordinary clothes; how clothing goes on is open. [early] (conv:141)

### Interface look
- One interface shell for HUD, overlays, dialogue, barter, inventory and manager, in first person and strategic mode; the HUD is "not gonna be a minecraft hud". [decided] (vision.md:184; conv:252-254)
- UI art at 640×480 (perhaps 1280×960 later), integer-scaled to any screen, bevelled borders, restrained colour, 256 colours; the 3D world at native resolution. [decided] (vision.md:185; decisions.md:427-428; conv:372)
- The HUD's bottom bar: the equip section on the left, three glass spheres (blue energy, red blood, green stamina) in the centre, the quickbar on the right. [decided] (vision.md:186-187; decisions.md:493-495; conv:372)
- Widescreen presentation decided once the shell runs. [wanted] (decisions.md:492)
- The spheres are 64-pixel circles holding a sloshing pixel-fluid sim, read by feel rather than as bars or numbers; eye candy people shake the mouse to watch, yet light to run: "you may be reading it by feel sincs its not a bar or number". [idea] (vision.md:434-436; conv:372)
- The fluid shows events: blood splashes; crits crack the glass with leaking rivulets; regeneration drops condense (faster with a buff); potions react in the pool; lightning arcs turn pixels to fire and smoke; magic sublimates blue; status effects change the fluid. [idea] (vision.md:437-453; conv:374-376)
- An equipped fireball's icon runs a flash of light that also shows its cooldown. [idea] (toy-factory.md:66-69)
- From opening the game to starting one: nostalgic for crafted, atmospheric 90s and early-2000s fantasy games. [idea] (vision.md:687-689)
- A fantasy, crisp DOS-style UI: menus and product art palette-crushed and hand-dithered, single-pixel 3D borders, 8-pixel button text like iron letters set in stone, with rust stains (after studying Diablo II's 144 colours): "looks exactly like an iron letter in a stone". [early] (conv:260, 274; from-grok.md:113-116)
- Unantialiased pixel art and bitmap fonts throughout; a 48-pixel decorative title font shaded by hand. [early] (conv:264, 278; from-grok.md:117)

### Sound
- A block's material sets its sounds; step and landing sounds also make noise for stealth. [decided] (block-properties.md:18; decisions.md:251, 862-863, 874-875)
- David makes all the sound himself (Omnisphere, his mics, an RME Fireface): layered footsteps, fire from the recorded burning piano, many variations of every repeated sound. [idea] (vision.md:927-931)
- No spoken words: David's own grunts and hmms; each kind of person or creature could have its own sound. [idea] (vision.md:932-933)
- Birds sing and flit about at the start. [idea] (vision.md:730)
- Music wish list: smooth jazz from Macroblank, Oblique Occasions, or Emancipator & 9 Theory: "a long ways off and dependent on how much buzz we can generate". [idea] (vision.md:934-935)
- The audio direction beyond these. [open] (vision.md:1005; regroup.md:49)

## 4 Performance and platforms

### Platform
- A commercial Steam build for PC, Windows first (Windows only for now). [decided] (vision.md:18; decisions.md:907-908, 968-970; regroup.md:10)
- Reference machines: an RTX 5070 Ti laptop and a 980 Ti desktop. [decided] (regroup.md:10)
- No per-card tricks: portable software on any card wgpu supports (980 Ti, AMD, Steam Deck); shared optional features only with a fallback: "id have to own 20 gpus". [decided] (decisions.md:1373-1377; dyno.md:1022)

### The targets
- It must run like a top, or the project can fail. [decided] (decisions.md:42-43; projection.md:21)
- A 980 Ti is the price of admission at the standard look: a smooth 60 at 1080p on a clean machine, and every card from there up gets that look: "980ti should be the price of admission at standard looks". [decided] (decisions.md:1532-1535; regroup.md:27; WG:1776-1777)
- Integrated graphics are measured for information, not designed to: "i don't expect someone to be able to play this game on an igpu without turning some settings way down. the 980ti is a different story". [decided] (decisions.md:1514-1527)
- Stronger desktops: 250+ without question. [decided] (decisions.md:984-989)
- The heaviest moments (a boss fight while directing two generals of spearmen) may dip below 240 as long as 120 holds. [decided] (decisions.md:1523-1528)
- 240 isn't holy: a budget readout shows when the wall is coming, so art and design can plan round it: "i don't want to treat 240fps as holy". [decided] (WG:1794-1796; msgs:216)
- The performance budget has to last. [decided] (WG:2428-2429)
- Keep headroom on a current, not super-powerful PC. [wanted] (proposal:869)
- Card memory well below 4 GB in the worst case; it may grow with the world's size (giant modded worlds are their makers' responsibility), never with play: "as the player plays the game for days, and weeks, and months, that the burden of memory continues to increase without limit, that's a serious concern". [decided] (decisions.md:1244-1253)
- Solid at a player's pace; fast developer flight may creak: "if the game creaks at 10,000mph that's fine. as long as its not aggregious and it's solid as a rock when i stop and act more like a player". [decided] (decisions.md:1254-1256)
- What goes into the "standard look". [open] (decisions.md:1535)
- (The laptop's target: see To settle.)

### How it feels
- Stutter is a dealbreaker: "its stuttering and thats a dealbreaker." [decided] (decisions.md:976; regroup.md:16)
- Even frames beat fast frames; even choppiness beats irregular lag; the work is predictable every frame. [decided] (decisions.md:1408; regroup.md:782, 1288; mem/speed-deep-dive-later:21)
- Response rate (input to photon) is a first-class measure, alongside frame time: "focusing on our 'response rate' instead of framerate first". [decided] (decisions.md:1052-1055; regroup.md:168)
- A frame cap just under the screen's top rate (237 on a 240 screen); if it can't keep up, stop and assess. [decided] (decisions.md:1063-1064; regroup.md:253)
- Frames paced by the screen's refresh while the simulation clock stays independent; every job carries a priority, as in Dwarf Fortress: "a criticality flag system almost like df has job priority". [decided] (decisions.md:1031-1042; regroup.md:300)
- The body and edits never give way under load; the standing camera is exactly still. [decided] (regroup.md:288; far-view.md:176)
- Under load, simulation steps are skipped farthest and lightest first (World, then Intermediator, Local last), as pauses that never lose state: "theres nothing wrong with npcs drawing a blank". [decided] (decisions.md:1047-1051; regroup.md:300)
- Flying fast may delay detail, but the frame never waits on the world; soaring over the whole world stays smooth; outrunning streaming above about 2,000 mph is accepted; players won't move faster than about 200 blocks a second. [decided] (decisions.md:1509-1514; WG:1769; far-view.md:610; proposal:366)
- Camera wobble and vibrating blocks far from the world's middle: set aside and written down: "it could cause problems later and we'd forget what it was". [open] (watch-list.md:461-462)

### How the machine is used
- The processor's drawing work stays at the floor; everything drawn is the card's work; the processor is for the simulation: "ONLY write code that increases the gpu's effort". [decided] (decisions.md:1569-1593; far-view.md:829; WG:2330-2331)
- The processor never does in a frame what the Steam build wouldn't. [decided] (regroup.md:9; decisions.md:968-970)
- Cost follows the screen, not the world. [decided] (decisions.md:1499-1500, 1585-1586)
- A stage that takes whatever is put on it, so performance can be forgotten: "i need to forget performance is a thing at all". [decided] (decisions.md:1548-1556)
- Nothing relies on fog to perform; the renderer is tuned for the clearest day: "nothing should rely on fog in our renderer to perform". [decided] (decisions.md:1397-1401; dyno.md:511)
- Hardware relaxed but ready: zero wake-up latency, no busy-spinning unless measured, running cool: "relaxed but ready to act instantly without any distraction". [decided] (decisions.md:1043-1046; regroup.md:239)
- In front, the game takes the PC's full attention, but alt-tab to Discord stays quick: "as long as people can at least tab back to discord quickly". [decided] (decisions.md:1065-1068; regroup.md:242)
- The game freezes deliberately when not in front: one greyed "Paused" frame, the simulation stopped, a click to resume (a setting): "i'd rather have the game just freeze (deliberately)". [decided] (decisions.md:1059-1061; regroup.md:246)
- Use idle hardware, the texture units and rasteriser first; no idle lanes; the processor works one way and never waits on a round trip: "we need to look into ways to exploit texture units and rasterizers". [wanted] (decisions.md:1305-1334; dyno.md:685)
- His speed ideas: mouse anticipation sent ahead from the processor to the card; specialised data slots; far pixels refreshed only once they've moved; screen tiles sorted by type. [idea] (decisions.md:1253-1271, 1335-1352)
- The lite simulation might run on the card: "the gpu could do a better job". [early] (conv:103)

### Streaming and the first bake
- The far levels are baked once per world from the real recipe; front-loading is accepted: "brute forcing things into cache in the beginning is an acceptable cost". [decided] (decisions.md:1139-1145)
- About an eighth of the world (about 8 miles round the spawn) is present at once while the rest fills in; the starting reach scales with the card. [decided] (decisions.md:1153-1163; regroup.md:483)
- Players get a loading screen while the first part bakes; a start-up burst to the thermal limit is fine behind it. [decided] (decisions.md:1165-1168; regroup.md:552; far-view.md:896)
- Chunks load nearest and ahead first, spreading the work rather than spiking; chunks left behind are dropped, changed ones saved; chunks made on the fly in the background, fast, without architectural compromises. [decided] (WG:1774-1775, 1793-1794; regroup.md:726; msgs:213)
- At least 32 chunks of full-detail gameplay round the player, city areas perhaps more; render distance separate from simulation distance: "at least 32 chunks of full fat gameplay around the player". [decided] (WG:1790-1792; msgs:216; conv:442)
- One long first wait to spare players chunk loading, perhaps with a tiny Hazard Course world in front: "i am willing to make the player endure a long initial 1 time wait". [idea] (vision.md:713-716)
- Real chunks only where the Local simulation runs. [idea] (simulation.md:236)
- The bake's catch-up "creaking" was felt; baking's own mark. [open] (dyno.md:770)

### Testing on other machines
- Weak hardware is tested by a pre-alpha of about 10 players, fun enough to play, who send back a hardware and performance report file: "a pre-alpha build fun enough that 10 people can play". [decided] (decisions.md:1612-1626; watch-list.md:496-501)
- Accepted risk: nothing tests weak hardware until the ten report. [decided] (watch-list.md:521-522)
- What "fun enough" means, who the ten are, what the report holds and how it's sent. [open] (watch-list.md:503-505; decisions.md:1627)

## 5 The player

### Who you are
- You start in year 1000 as a character with no known origin and little to your name. [decided] (WG:2151)
- The same rules for NPCs and the player, plus a noticeable player edge in efficiency, drops and speed over NPC miners of the same skill. [decided] (decisions.md:275-278)
- The chosen-one framing explains carrying five beds, abilities NPCs lack, and NPCs reacting differently to you. [idea] (vision.md:63-67)
- The God of Fire's son, 18, origins unknown, waking in a tent on a road (his mother: see To settle). [idea] (vision.md:50-51)
- (Where you start: see To settle.)

### The start
- You wake crouched in your tent (crouch-only; you can sleep in it) with a diary, a burlap sack and a magical loaf. [idea] (vision.md:466-468)
- The magical loaf appears once: rainbow sparkle, every bite heals and restores fully, stale in a week (an ordinary loaf has 4 bites). [idea] (vision.md:469-471)
- You never pick a spawn: you see the world once in the intro, then work out where you are by talking or exploring. [idea] (vision.md:708-711)

### Body and races
- Races: dwarf 1.5 m, human 1.7 m, elf 2.1 m; elves crouch through human and dwarf doorways. [decided] (decisions.md:202; vision.md:226)
- Each race balanced to be fun to play; humans the standard choice: "focus on humans as the standard choice". [idea] (vision.md:33; conv:529)

### Movement
- Minecraft-like controls for now: WASD, mouse look, Space to jump, left click breaks, right click places. [decided] (decisions.md:901-902)
- Three gaits: Shift runs, double-tap Shift sprints (forwards only), Ctrl crouches. [decided] (decisions.md:849-850, 869-871)
- A clean jump clears a 2-block gap from a walk, 3 from a run, 4 from a sprint. [decided] (decisions.md:849-850)
- Sprinting widens the field of view by 10°. [decided] (decisions.md:873; conv:480)
- Movement shaped by ADSR envelopes on a draggable graph: sticky and decisive, with sharp starts and ends that read clearly, important for combat. [decided] (decisions.md:847-848, 876-878, 888-889; vision.md:526)
- Momentum: a charge built while running, spent by sliding when you stop; the "woah woah woah" stop is meant to be a skill; no special backpedal braking. [decided] (decisions.md:854-859)
- Sideways and backwards slower than forwards. [decided] (decisions.md:883)
- A visceral parkour feel: sprint freezes in mid-air; minimal but not zero air control; you can swing arms and legs to try to mantle. [decided] (decisions.md:884-886)
- Realism comes from self-similarity, not slow walking (no early-Morrowind drag); speeds at Minecraft defaults for now. [decided] (decisions.md:890-891)
- Per-block friction (ice is slippery). [decided] (decisions.md:879-881)
- You step up half blocks. [decided] (quartile.md:178)
- Crouch: a shorter body, lower eyes, slower; you can't stand without headroom, and won't step off edges (switchable). [decided] (decisions.md:865-866)
- Fall damage and stumbles. [decided] (decisions.md:862-863)
- One shared kinematics layer for players, NPCs, animals, carts and falling trees. [decided] (decisions.md:860-861)
- Mouse feel changed on purpose for states: a jolt when hit, drift when sick, the view sagging when you've been awake too long (with a yawn to explain it): "having the mouse drift downwards when youve been awake too long would be fun". [wanted] (decisions.md:1353-1368)
- Later: mantling (maybe), a landing slide, a jump lift-off envelope; Athletics, Agility and Speed modulating the start of a sprint. [wanted] (decisions.md:851-853, 872, 882, 887)
- The Windows Sticky Keys shortcut switched off while the game runs. [wanted] (decisions.md:867-868)
- Players can't fly in normal play: "i don't forsee players being able to fly". [early] (msgs:213)

### Stats and the three spheres
- One RPG system for NPCs and the player: Str, Con, End, Int, Wil, Cha, Spd, Dex, Agi, with skills such as Athletics. [decided] (vision.md:247-248)
- Wil is willpower: "willpower probably, since wisdom is ambiguous against intelligence". [decided] (vision.md:307-308)
- No "health": three spheres, blood (red), stamina (green) and energy (blue): "they represent your blood, your stamina, and your energy". [decided] (conv:370; vision.md:433; decisions.md:493-495)
- Attributes by sphere: red Str, Con, End; green Agi, Spd, Dex; blue Int, Wil, Cha. [decided] (conv:370; vision.md:296-301)
- How skills improve, and where attributes come from. [open] (vision.md:999)

### Injury, needs and fatigue
- Damage is blood loss, injuries and effects: blows take blood at once plus bleeding over time; blunt hits don't bleed but take blood, stamina and energy at once. [idea] (vision.md:438-440; conv:370)
- Critical hits crack the glass; bandaging stops bleeding; natural regeneration; health potions. [idea] (vision.md:441-445)
- A wide range of injuries, each with its own impediments; you get grievously hurt quickly. [idea] (vision.md:446-447)
- Stamina drains as you act; some recovers, some is lost until you sleep. [idea] (vision.md:448; conv:370)
- Energy pays for mundane things and for magic; spells with gradients reach full effect only with a full sphere. [idea] (vision.md:449-451; conv:370)
- Energy is also sharpness: two nights awake worsen charisma and intelligence checks. [idea] (vision.md:546-547)
- Hunger impairs heavily and takes no blood itself, but starvation can still kill; hungry, you lose more blood when hit and bleed longer: "the hunger has to be bad with lots of impairments". [idea] (vision.md:345-354)
- Healing comes from physicians and from magic: "physicians and magic should both be methods of healing". [idea] (vision.md:270)
- The injury list and its impediments. [open] (vision.md:997)

### Death
- With followers: the camera on the ground, a fade to black, a companion's cry ("[name]! No!"); followers fight on; you wake in a shrine or with them, and the world sim decides whether they escaped. [decided] (simulation.md:70, 84-86; vision.md:459-461)
- Death: you wake in a church or ruined shrine of a god, barely alive, with some things missing. [idea] (vision.md:457-458; simulation.md:89)
- A hardcore permadeath option, later. [idea] (vision.md:462)
- What happens to your settlements in hardcore. [open] (vision.md:996)

### Inventory and carrying
- Few inventory slots, extended by backpacks, followers, carts and pack animals; weight and volume; containers. [decided] (vision.md:803-804)
- Fair inventory balance, not strict realism; carrying several beds in the hotbar stays. [idea] (vision.md:31-32)
- One slot holds a material in every size, counted by volume: "that could be 16 whole blocks, 64 quarter blocks". [idea] (vision.md:806-813)
- Every item is an inventory of like items (stacks). [idea] (vision.md:383-392)
- Ten slots to start; a backpack takes one and holds about 6; books stack to 4 even when unique. [early] (conv:378; from-grok.md:84)

### Mounts and vehicles
- Mounts and vehicles (a donkey, a two-horse wagon, a mule cart, magic running shoes) under one physical feel. [wanted] (decisions.md:892-893)
- Mounts, livestock and hunting in detail. [open] (vision.md:1002)
- Pack donkeys essential for an adventurer: a donkey train follows your steps; carts on trails: "donkeys with packs become crucial for an adventurer". [early] (conv:24, 529)

### Personal screens
- A personal screen with tabs for memorised spells, status effects, armour and stats. [idea] (vision.md:552-553)
- Screens for crafting, armour and stats, levelling up, and a spell list whose spells slot into the hotbar; a "personal/party" inventory screen. [early] (conv:244, 378)

## 6 Survival, food and crafting

### Food and nutrition
- Food rewards with a cozy feeling for players who roleplay; the coziness comes from names and looks, not sentimental text: "food should reward with a cozy feeling". [wanted] (vision.md:398-401, 417-418)
- Food colours feed attributes: red (protein: meat, beans, nuts, eggs) Str, Con, End; blue (vitamins: fruit and vegetables) Int, Wil, Cha; green (starch, carbs, fat) Agi, Spd, Dex: "i see red as being meat, or protein, or beans/nuts/eggs". [idea] (vision.md:296-301; conv:370)
- Fat and carbs are both fuel and refill the same meter: carbs spike fast and wear off quickly, fat wears off slowly: "they should both refill that nutrition meter". [idea] (vision.md:280-286)
- The food colours aren't explained; the item card names the food and what's in it: "as long as food is labelled when you check the card". [idea] (vision.md:311-313)
- Most food and farming comes over from Dwarf Fortress. [idea] (vision.md:279)
- Food is a fun loop in its own right: "fun loops will attract lots of people with shorter attention spans". [idea] (vision.md:355-356)
- Many ways to food, enough to carry a player through a year. [idea] (vision.md:328-330)
- Storing food properly prevents spoilage, which "makes you find and fire clay". [idea] (vision.md:343-344)
- "Nutrition" can't be both one part and the name of the set: names, or just colours: "i could just color them with no names". [open] (vision.md:283-288)

### Cooking
- Complex cooking always pays slightly more than the sum of its ingredients: "cooking more and more complex food should always pay off". [idea] (vision.md:341-342)
- Almost cutesy cooking minigames, if the reward is enough. [idea] (vision.md:379-381)
- Foraging wild plants and cooking them by many recipes with fat, plants and herbs. [idea] (vision.md:336-337)
- Curing meat takes 2 herbs or 1 salt; herbs add nutrition, salt none; salt has other, non-critical uses. [idea] (vision.md:314-317)
- Cured meat is better balanced than plain meat but weaker overall; a cured haunch is less nutritious but keeps a while. [idea] (vision.md:320-322, 423, 428)
- Spits use sticks (3 for a haunch, 1 for a bird, fish or rabbit), shown in the campfire until reused or removed. [idea] (vision.md:419-425)
- A roast haunch is about 4 meals and spoils in 2 days; smoking keeps longer but needs a smoker and more wood. [idea] (vision.md:421-427)
- Knives by job: a chef's knife for preparing ingredients, a regular knife for skinning and quartering, a butcher's knife for butchering. [idea] (vision.md:381-388)

### Hunting and butchery
- Hunting a thrill, with spear throwing, arrows and traps for rabbits: "hunting animals should be a thrill". [idea] (vision.md:338)
- The first animals: deer, fish, rabbits, fowl, pheasant, boar, giving meat, hide and fat. [idea] (vision.md:339)
- No organs: "i dont think organs are necessary, theyre all just meat". [idea] (vision.md:339-340)
- A carcass pops apart into stackable pieces, one kind of piece per animal (a haunch). [idea] (vision.md:382-400, 420)
- Roast a piece whole (easy, hard to store) or butcher it into slices or steaks (more meat, more ways to cook, easier to store). [idea] (vision.md:383-396)
- Campsite trades: a fishing spot near fish, a deer skinning and tanning rack; hunting and logging camps. [early] (conv:24)

### Farming and foraging
- A solo player can farm without ever defining an area. [decided] (decisions.md:357)
- Growth: dirt types and what grows on them. [wanted] (block-properties.md:29)
- Crops slightly time-intensive but very rewarding for a player alone: "extremely rewarding to the player minecrafting alone". [idea] (vision.md:327-328)
- Gathering flowers, incenses and plants for spells of fasting. [idea] (vision.md:335)
- Farming: barley or wheat, a hoe, a scythe, seeds, beds. [early] (conv:192)

### Fire and light
- Torches are needed; David builds and registers torches and wall lights himself in the Registrar (placing on walls still to come). [decided] (decisions.md:1452-1453; objects.md:344-346)
- Fires as in Songs of Syx: small (1 quarter log and sticks), larger (3 small logs and sticks), largest (1 large log, 6 small logs and sticks); the quarter oak log is the fires' small log. [idea] (vision.md:477-480; quartile.md:14)
- Cold NPCs want to warm up. [idea] (vision.md:481)

### Mining and stone
- Digging is physical and finite, with the same rules for the player and miners: "that's the way". [decided] (decisions.md:261-265)
- Staged mining: rock, then boulder, then a boulder with a pick or a full block with hammer and chisel; speed depends on tool, material and skill: "stones are pennies and boulders are nickels". [wanted] (decisions.md:259-260; vision.md:804-805; conv:392)
- Hardness sets mining time and tool types or tiers; RPG skills affect mining speed. [wanted] (block-properties.md:21; decisions.md:259-260)
- Cutting stone on screen with a chisel on the ground; the minigame can knock an octant off a slab: "done on screen with a chisel on the ground". [idea] (vision.md:822-824; quartile.md:444, 531)
- Three rungs of getting ore: a little Minecraft-style, a fair amount by strip mining, huge amounts from big mines with many workers. [idea] (decisions.md:317-323)
- A block mines into 2 boulders and a rock; a boulder into 3 rocks or one cut stone; ore drops piece by piece, more for skilled miners; each moment of mining takes a share of the block's HP. [early] (conv:392; from-grok.md:89-91)
- Dirt is easy to dig for the player and NPCs. [early] (conv:390; from-grok.md:95-96)

### Trees and wood
- A felled tree falls physically and turns over whole (a quarter turn), as one body, landing as a flat stack of logs; the branch underneath folds aside before it lands. [decided] (decisions.md:604-626; proposal:81)
- The fall direction is slightly random: a roll at the cut, within a range, so clumsy villagers or players can get hurt: "if you roll below a theshold it may fall to the back or side". [decided] (decisions.md:627-631)
- Leaves burst on any collision, "like shooting a goosedown pillow"; only leaves with no room vanish; branches on the side it falls toward break off. [decided] (decisions.md:604-606, 622-626)
- Logs are processed on sawhorses and workbenches (sawn at a sawhorse, sawmill or carpenter). [decided] (vision.md:804-805; conv:378)
- Tree felling is a mechanic long wanted; a tree-felling mod idea set off the whole project. [wanted] (decisions.md:582-583)
- Felling leaves the logs together; you chop them up again in a minigame, into quarter logs for fires or building; a cut piece pops off and can be picked up: "it has to pop off and be pickupable". [idea] (decisions.md:599-604)
- The fall tips over an edge, speeds up, drops and may bounce once; it can injure and knock back players; it looks bouncy but snaps each piece to the octant grid, and has to be fun to watch. [idea] (decisions.md:609-610, 635-636)
- Leaves pile on half the logs. [idea] (decisions.md:607-608)
- Only the bottom block can be cut; leaves give sticks; big and small logs; big logs quartered by standing them up and chopping: "the trees bottom block is the only mineable one". [early] (conv:388, 404-408; from-grok.md:92-94)

### Tools, weapons and quality
- Alloys, with recipes: bronze, hard bronze, black bronze, brass, sterling silver, iron, steel. [decided] (vision.md:630)
- Stone tools: crude flint or stone tools nobody would buy, needed if you leave the path you wake on. [idea] (vision.md:474-475)
- The club is the first weapon you craft. [idea] (vision.md:476)
- An item's strength comes from quality, material and encrusting: "item quality, material, encrusting will make a big deal to the items strength". [idea] (vision.md:866-869)
- Quality decides most of an item's worth within a material; the material sets the range: "poorly made adamantine or mithril weapons will still obviously be better". [idea] (vision.md:883-891)
- Adamantine and mithril exist as top materials. [idea] (vision.md:884-885)
- Sharpness isn't durability: it wears down fairly fast and comes back fairly easily. [idea] (vision.md:885-892)
- Skill makes a better edge that lasts longer and hits harder; the material sets how many steps of sharpness a blade holds; dull weapons lose effectiveness only down to a floor. [idea] (vision.md:886-899)
- A whetstone sharpens slowly, a step at a time; a grinding stone fast. [idea] (vision.md:887-896)
- Items show the material they're made of. [early] (conv:16)

### Crafting and workshops
- Recipes (what's made, from what, at which station, how long) are content David provides. [wanted] (jobs.md:169)
- Crafting by list, not a grid (no Minecraft "tic-tac-toe box"). [idea] (vision.md:857-858)
- Two kinds of crafting at one set of stations: the player walks up and makes a thing (Skyrim style); villagers labour in chains (Dwarf Fortress style). [idea] (vision.md:852-856)
- Workshops of any size (Songs of Syx); perhaps Vintage Story-like crafting inside, without drifting too far from Minecraft. [idea] (vision.md:835-839)
- What makes one workshop different from another. [open] (vision.md:838-839)
- Production shown physically, step by step (log, sawhorse, planks, table, furniture, warehouse shelf); bakeries, kitchens and blacksmiths the same: "bakeries, kitchens, blacksmiths, will all work this way, creating interesting behavior." [early] (conv:380)
- Alchemy laboratories and potions; a healing potion is a purple phial. [early] (conv:246, 374)

### Storage and hauling
- Pallets show stored blocks and stone in miniature. [decided] (decisions.md:243)
- Resources are physical and take up space; "haulable" says how a thing must be carried or stored (wheelbarrow, cart, barrel, bag): "resources are physical and take up space". [wanted] (block-properties.md:20, 25)
- Container blocks with inventories. [wanted] (block-properties.md:30)
- Heavy, physical items push you toward labourers, wagons, carts and donkeys; needing a wagon and two donkeys for a huge load is fine; tangible storage is fun to look at. [idea] (vision.md:28-30; conv:378, 529)
- Pallets (1×1 holds 4 small stacks, 2×2 holds 16; quarter blocks 8 to a pallet block), crates, bins (open 2×2 for boulders), boxes, chests, coffers; casks, barrels and jars hold one kind each, and a barrel of cheese counts as a food item. [early] (conv:16, 392-394; from-grok.md:85-86)
- Items can be stacked on the ground. [early] (conv:16; from-grok.md:87)

### Items and drops
- Drops work as in Minecraft unless something better comes up: "if i cant think of a better way, minecraft isnt broken". [decided] (decisions.md:637)
- Do drops fade after a while? [open] (decisions.md:646-647)
- How much object variety the world has; clutter or real items, judged by fun; whether things can be picked up: "a can of worms". [open] (vision.md:859-860; watch-list.md:409-414; objects.md:41, 85-86)
- Item kinds: Food, Materials, Supplies, Tools, Weapons, Armor, Valuables, plus books (where do books go?). [early] (conv:316; from-grok.md:88)

## 7 Building and materials

### Blocks
- Minecraft-scale blocks with 32×32 faces. [decided] (vision.md:802)
- Three kinds of thing: blocks, objects and blueprints: "blocks are blocks, objects are objects, and blueprints are made of both". [decided] (decisions.md:670-676)
- A block is material × inclusion: the material gives hardness, sound and colour; inclusions are an overlay; texture variants allowed later. [decided] (decisions.md:251-253; vision.md:626)
- A "constructed" bit, plus richer ownership (pillagers, quest tags) for the few blocks that need it. [decided] (decisions.md:347; block-properties.md:24)
- Blocks carry a see-through medium (a colour and a clarity): water, glass and ice. [decided] (regroup.md:1140, 1252)
- Fewer than 65,000 block types; a few thousand is plenty. [decided] (decisions.md:350)
- David's block properties: material, inclusions, yield, hardness, structure (gravity and support, later), placement, constructed, haulable, building ownership, light (emission and opacity, later), temperature, growth, inventory, discovered, seen. [wanted] (block-properties.md:16-32)
- A name for the "what kind of thing" dimension (natural granite against cut stone against brick). [open] (decisions.md:254)

### The quartile system
- Each cell splits into 8 octants; a quarter block is a 1 × ½ × ½ bar along any axis (12 slots per cell); bars can cross two cells and mix materials; it is among the most critical pieces: "the placer and quartility seem the most critical". [decided] (decisions.md:224-228; quartile.md:5, 283; WG:2233)
- Quarter blocks replace stairs, slabs and roof pieces; dirt slabs exist. [decided] (decisions.md:228-229)
- Only blocks marked in the Registrar can be bars; David registers his own. [decided] (quartile.md:178)
- Every octant is a stick-on, and everything has collision. [decided] (quartile.md:194, 374)
- Octant items are objects (a cup, a candle); lone octants ("octblocks") exist. [decided] (quartile.md:191; vision.md:822-824)
- Mixed-material cells: the first bar placed keeps a shared octant. [decided] (decisions.md:238-239)
- Mining: a mixed cell is destroyed whole and blocks drop less than they're worth; the player gets a one-material shape back, mixed odd shapes drop nothing; cut pieces are never retrievable; single bars are mined one at a time; mining a lone octant is acceptable. [decided] (decisions.md:238-239; quartile.md:199, 290, 356, 386)
- L-shaped pieces as blocks, if needed. [idea] (quartile.md:531)
- (Whether bars may overlap: see To settle.)

### Placing
- Every exposed face offers four ½ × ½ quarters and a centre spot; picture frames hang on wall quarters; small items take a quarter or the centre, large items need a full flat face (or two). [decided] (decisions.md:230-237)
- Nothing goes on an occupied surface; objects placed against blocks protect them from mining. [decided] (decisions.md:240-242)
- The placing controls are his to design: B cycles sideways, longways and upways; the left or right half of a face flips direction; Shift frees a block from the grid onto the octant grid. [decided] (quartile.md:188, 360, 433, 473; handoff.md:193-201)
- B turns an object a quarter turn: "rotation might as well be B". [decided] (objects.md:217)
- The placing helper: a ghost of the placement; the selector snaps to quarters and shows rotation; the exact octants highlight; a magenta box to place, a cross on the face to dig; a floating cone marker at the aimed face or quarter. [decided] (quartile.md:91, 283, 315, 357-358; regroup.md:982; decisions.md:245)
- Digging and placing show at once, near and far, at every distance. [decided] (handoff.md:30; proposal:59; regroup.md:982)
- On-screen placing helpers that can be dimmed likely stay in the finished game: "will probably stay in some form in the end game". [wanted] (objects.md:217-218)
- Items on faces and in quarters (mugs, frames, spears). [wanted] (proposal:86)

### Objects
- Objects are placed only by blueprints and players: "only blueprint stuff and player stuff". [decided] (objects.md:72-73)
- Objects include swords, armour, braziers, torches, bread, cups, plates and beds: "these are all the same thing, swords, armor, brazieres, torches, bread". [decided] (objects.md:3-7, 142-146)
- An object's size comes from its model: wider than 8 pixels it snaps like a full block, 8×8 or smaller it sits on octants; it may stick over a little and clip. [decided] (objects.md:190-216)
- Collision follows the model's geometry, not its volume. [decided] (objects.md:219)
- A 64×32×32 bed must be placeable. [decided] (objects.md:216-217)
- The brazier: Doom-style, modelled in Blockbench, solid, centred in its cell, standing on the floor, with its own light (a green flame), saved and collidable. [decided] (objects.md:144-145; regroup.md:1098; caves.md:485)
- Moving block objects: a felled tree, carts, boats, doors. [wanted] (proposal:81)

### Blueprints, structures and zones
- Blueprints: trees, buildings and player blueprints are one kind of thing. [decided] (decisions.md:667-669)
- Structures are defined by building them in the suite, with invisible "imaginary" hook blocks seen only there; the first structures were oak trees. [decided] (decisions.md:576-578; proposal:84)
- A structure editor (the "temple") with data-point blocks for wave-function-collapse village pieces and tree growth. [decided] (vision.md:803; decisions.md:562, 574-575)
- Player blueprints up to 32×32×32, stamped together in segments and turned 90° for huge builds. [decided] (decisions.md:742-744)
- Workshops are blueprints of stations; villagers do visible multi-step labour. [decided] (vision.md:833-834)
- Zones are the primitive: build something, zone it if it meets a room's criteria, save it as a blueprint if you wish; a blueprint carries its zones; rooms built freely, then zoned, or from a blueprint. [decided] (decisions.md:709-712; simulation.md:380, 387)
- Zone types live in the Blueprints tab, edited in a zone designer (seats per diner, a dance-floor counter, storage). [decided] (decisions.md:713-715)
- Areas are stored as shapes: selecting a room, an area or an in-mountain "cell" marks it as warehouse, pasture, meeting area and so on; warehouse capacity is what you build. [decided] (decisions.md:354-356; vision.md:833-834)
- Player blueprints: nothing forbidden; problems are flagged. [idea] (decisions.md:773-775)
- 4 or 5 area zones without a blueprint (a pasture with its fence and gate), each made its own way, with a script. [idea] (decisions.md:700-704; simulation.md:394)
- Baked paths in structures, precomputed and tried first, but the blocks are the truth: "a cheap network of things that can get prebaked". [idea] (decisions.md:755-768)
- Past a "% changed", a building stops being a building. [idea] (decisions.md:771-772)
- Marking paths is separate from making roads; a line tool for roads, paths and stairs in settlement mode; an invisible string for followers. [idea] (simulation.md:412, 435, 441)
- How a pasture gets its fence, gate and barn. [open] (decisions.md:701-702)
- Blueprints both negative (dig) and positive (build); templates stamped down as orders, with haulers filling a warehouse near the build: "a blueprint system that is both negative and positive". [early] (conv:46, 182; from-grok.md:103)
- Workshops and structures level up from blueprints (a bakery needs tables, shelves, ovens), because decorating is fun in Minecraft and clumsy in Dwarf Fortress. [early] (conv:30; from-grok.md:22-23)

### Structure and limits
- Nothing should stop tall structures: "nothing should stop a Tower of Babel". [wanted] (WG:2434-2435; handoff.md:299-302)
- Structural gravity and support, later. [wanted] (block-properties.md:22)

## 8 Settlements, people and simulation

### The living world
- The world depends on NPCs acting of their own accord; they crave jobs, and bustle comes from simple decisions: "NPCs are creatures that crave jobs most of all". [decided] (jobs.md:6-8)
- You can't fully control NPCs; planning around their routines is part of the fun, and it's never fully mechanical. [decided] (jobs.md:10-11)
- The simulation is Dwarf Fortress-deep, on the processor, which has "its work cut out for it with the simulation". [decided] (decisions.md:1575-1576, 1591-1593)
- The simulation's core: tables that don't know what they hold, rules naming what they read and write; whatever replaces Bevy keeps smart parallel work, data-agnostic tables and the existing clock: "hardcoding types of tables is a bad idea, it should be completely agnostic". [decided] (regroup.md:1432; decisions.md:1537-1547)
- The aim: a background spreadsheet David can steer, yet emergent and too complex to fully predict. [idea] (vision.md:985-989)
- Richer villagers, not more of them: each life counts; rich NPCs at party scale, not mobs. [idea] (simulation.md:217-222)
- Villagers act like Minecraft's first, then sophisticated, then unbalanced to taste; the work order: villages and people, jobs, time, balance, sophisticate, unbalance. [idea] (simulation.md:192)
- Rules over arbitrary data fields as formulas (dice algebra), nothing hard-coded; many behaviours from few impulses; at ship time the rules tangle, crazy just often enough, never game-breaking. [idea] (simulation.md:202, 211, 325; conv:398)
- Do agnostic tables stop at new columns, or reach new rules written in data? [open] (decisions.md:1543-1544)
- Villagers eat, so there is always activity. [early] (conv:22)
- Visible routines: working, eating, sleeping, socialising, drinking, worshipping, idling; fetching and hauling, scaffolds, hunting, logging. [early] (conv:326)
- Villagers sleep at night and spend half the day trying to work and half wandering. [early] (conv:192)
- Cheap movement: walk in a direction outside and re-route only when stuck. [early] (conv:330)

### Three levels of simulation
- Three levels: Local (full, round the player and his settlements, 30 ticks a second), Intermediator (hourly; prepares places for unfolding) and World (daily, a lite "spreadsheet of what's going on out there"); the same rules at different coarseness; one place, one level, with natural boundaries; hooks with expiry; journeys and Fate. [decided] (simulation.md:13-36; decisions.md:813; vision.md:651-655; handoff.md:261-264)
- Everyone is named from the very beginning; detail unfolds as you come near: "evey person in the world is named from the start". [decided] (jobs.md:139-140; simulation.md:57; conv:543)
- The full simulation drifting from the bookkeeping when you arrive is good, giving openings for events; story elements must agree exactly. [decided] (jobs.md:64-69)
- The player keeps places live through hooks (quests, residence, ownership, followers); a richer lite sim runs for places you own, live in or have quests in, and changes persist: "return from the quest in a month, theyll still be gone". [decided] (simulation.md:38; conv:539-543)
- Travelling groups converge on you as you near their route. [decided] (simulation.md:54)
- The world sim hands the local sim the world where you go, and takes it back when you leave. [idea] (vision.md:666-667)
- Full-detail NPCs only within the simulation radius; the rest updated daily or weekly. [idea] (simulation.md:225)
- People you know can die in accidents while you're away. [idea] (simulation.md:240)
- The lite sim as a spreadsheet: harvests, daily food, starvation when a farmer dies, births and deaths of named people, children growing up, taking jobs, going on raids and dying: "but thats all lite, its a spreadsheet". [early] (conv:103, 545)

### Impulses and jobs
- Impulses (work, eat, sleep, socialise, more later) charge up and become the active goal at a threshold; opportunities press the threshold down; randomness acts slightly on every impulse, with occasional "waves of weirdness"; self-balancing, so even a dwarf with several urges full eventually sleeps. [decided] (jobs.md:15-28, 70-71)
- Needs (work, eat, sleep, wander, socialise); happiness from work, rest and play. [decided] (vision.md:227-228; conv:370)
- Opportunities are very local pressures; general pressures can be broad (a biome, a season, a mood). [decided] (jobs.md:43-44)
- Different interactions satisfy different amounts (meeting someone disliked half-fills the need). [decided] (jobs.md:47-49)
- Chance meetings: villagers who meet daily stop to chat and become friends; away from the player, meetings are rolled rarely, as bookkeeping through Fate. [decided] (jobs.md:58-63)
- A Job is anything that needs doing, made by many sources: villages; kingdoms (couriers bringing news of Remarkable Events); goblin dens (heinous crimes); from orders, settings (a workshop keeping wood stocked) and opportunity. [decided] (jobs.md:75-84)
- Idle people take jobs on their own (they find pickaxes and dig). [decided] (jobs.md:85-86)
- Workers are assigned to workstations and check theirs first; each occupation has a short list of places to look before giving up, ending with a hauling job; those who give up look for other work, and the event log notes it. [decided] (jobs.md:92-113)
- People take disliked jobs when they must, with less satisfaction and more trips to the tavern; liked jobs make happy campers. [decided] (jobs.md:119-124)
- Hauling and warehouses get their own in-depth design later; simple hauling for now. [decided] (jobs.md:126-129)
- Content David provides: name lists per race, occupations and their search lists, recipes, impulses and their rates. [wanted] (jobs.md:165-170)
- Animals, monsters and wild animals that take jobs. [wanted] (jobs.md:75-76)
- Urges as meters that fill into desires; pressure classes lower the thresholds; criminality fills very slowly. [idea] (vision.md:975-981)

### Peoples and races
- Three "human" races (humans, dwarves, elves), all full people with urges, jobs, likes and friends. [decided] (WG:2126-2127)
- The other races (goblins and so on) are barbaric factions with simpler behaviour: threats like Minecraft's spawners writ large, but with purpose and narrative; with exceptions, such as a quest to talk to one unusual goblin in a unique neutral goblin village. [decided] (WG:2128-2133; conv:196)
- The lawful and chaotic, good and evil axes stay. [decided] (simulation.md:329)
- NPCs are recruitable and get assigned quarters and jobs; you recruit by dialogue ("follow me") and order planting by marking a rectangle. [decided] (vision.md:227-228; conv:192, 230, 262)
- Dwarves build underground, with great-hall entrances into the mountains busy with villagers going in and out. [wanted] (WG:1314-1315, 1343-1344)
- School for every child: "i think all juveniles can benefit from school". [idea] (vision.md:249-252)
- A college (leaning to alchemy) and a church (leaning to magic), each with its own kinds of both; no conflict built in, only specific or personal ones: "i like an idiosyncratic split". [idea] (vision.md:253-269)
- Guilds: "places for tradesmen of a type to gather and improve and socialize". [idea] (vision.md:271)
- Development starts with dwarves, since their settlement behaviour is the biggest addition. [idea] (vision.md:33-34)
- Race homes: elves in forests, dwarves carved into mountains, humans on plains. [early] (conv:16; from-grok.md:67)
- Lifespans: elves longest-lived and costliest ("like the protoss"), dwarves between, humans shortest-lived, quickest to breed and most adaptable. [early] (conv:171; from-grok.md:65-66)
- Dwarf women look almost like the men (narrower, thinner beards), are a third of births, and are a third less likely to want a job. [early] (conv:171; from-grok.md:59-64)
- Names from first- and last-name files (dwarven syllable sets given); dwarf sons take the father's surname, daughters the mother's; each race its own customs; human names Roman in flavour, elf names Hellenic. [early] (conv:97, 159-167, 364-366)
- Villagers carry one stack and hold one item; some jobs (cotton) need a bag. [early] (conv:16; from-grok.md:70)
- Research by knowledge workers in libraries, schools and laboratories. [early] (conv:246)
- Other peoples (lizardmen) may join your towns later. [early] (conv:256)
- Court or staff titles. [early] (conv:318)

### Settlements and growth
- Settlement types: campsite, cottage, farmstead, village, town, city, capital, outpost, fortress. [decided] (vision.md:178)
- Rooms and zones Dwarf Fortress's way: zones with key furniture, owned or shared, a bed claimed by sleeping in it, room value, pastures, locations with patron rules, guest rooms. [decided] (simulation.md:363)
- Resource outposts: small mining camps in the region next to a town; villages use them, fight over them, and find them contested by bandits or highwaymen. [decided] (WG:2117-2119)
- A quarry or mine built round a deposit lets a few employees produce steadily while the deposit is really there; tree farms where workers replant in rows. [decided] (decisions.md:261-267)
- Regions are the base unit kingdoms and factions operate in, sized carefully; a goblin den prevents building or expanding in its region. [decided] (WG:2110-2116)
- The placer places towns and their buildings, grown like jigsaw pieces and by NPC decisions, at world creation and any time later: "grow procedurally like jigsaw pieces and also through decisions by the npc". [decided] (WG:2243-2250; placer.md:351-356)
- Directing a settlement and soldiers from the orthographic view. [decided] (decisions.md:364-367)
- Visitor rules and locking doors, as in Dwarf Fortress. [wanted] (simulation.md:353)
- Below farmstead size, you manage through a diary that travels with you; larger needs a manager block; larger still, an advisor you talk to. [idea] (vision.md:181-183)
- A hamlet of about 20 people with 2 guards; the largest settlement about 500 (capitals, or the player's after about 3 months of building); 3 to 6 people per house, plus bunkhouses and inns. [idea] (simulation.md:219-235)
- Procedural puzzle-piece (wave function collapse) villages with their street networks baked; Riverwood in Skyrim as the model for the first one you find. [idea] (decisions.md:574-575, 776-778; conv:474; from-grok.md:80)
- Settlement borders announced; inhabitants under scrutiny for damage, theft and offence. [idea] (simulation.md:343)
- Inn travellers who work and may stay as inhabitants. [idea] (simulation.md:353)
- Management grows from campsite to cottage to farmstead through a management skill and management capital. [early] (conv:24; from-grok.md:14-15)
- Housing: campsite 4, cottage 6, farmstead 12; you travel easily with 3 followers. [early] (conv:30; from-grok.md:16)
- Perimeters that grow; outlying hunting and logging camps; roads. [early] (conv:24; from-grok.md:17-20)
- An abandoned settlement carries on crudely, withering or surviving: "it will just keep going and survive in a crude way". [early] (conv:24; from-grok.md:20-21)
- A site with no allies, followers or owned villagers drops out of the manager; you need a political stake to keep it. [early] (conv:30)
- Villages level and clear their own ground, easing slopes over about six blocks. [early] (conv:474-476; from-grok.md:78-79)
- One full management simulation at a time: you must be there to direct a place, and the rest run on your managers', nobles' and standing orders' skill: "you only have one full management simulation going at a time". [early] (conv:28; from-grok.md:30)
- Couriers or pigeons carry orders to your other settlements, with a delay. [early] (conv:312; from-grok.md:29)
- Warehouse windows reachable from the manager, the orthographic view, or the scribe's desk in person. [early] (conv:248)
- Sites against settlements: does a field of crops count as a structure? [early] (conv:314)

### Followers and inhabitants
- Inhabitants belong to a settlement and get orders; followers are your party, travel with you and can be left somewhere; you use their inventories and set how they follow. [idea] (vision.md:229-232; conv:256, 513)
- Followers are people in the world who join your party, directed more closely through dialogue. [idea] (vision.md:233-234)
- Followers stay close in a rough circle (like The Lord of the Rings, not Skyrim) and watch your back, so you feel the difference. [idea] (vision.md:240-243)
- Who can follow: leaning to a blend, a fighter or rogue in any village for a price, plus story followers from quests. [open] (vision.md:235-237)
- How much control you have over followers, at least a posture setting (sneaky, aggressive, attacking). [open] (vision.md:238-239)

### Economy and trade
- Construction and production orders cost gold as well as materials. [decided] (vision.md:608; conv:382)
- The economy is mostly for show: the Dungeonmaster makes it look busy with caravans and market stalls; economic pressure can drive township decisions; it's tuned much later, in a living world. [idea] (vision.md:602-609)
- For the player the economy is real: earning money for a pickaxe is as viable as smelting copper. [idea] (vision.md:606-607)
- Villages trade for what they lack; caravans spawn from need and surplus. [idea] (vision.md:660-661)
- Traders as small wagon parties on the roads, stopping at trading posts, met on the road; unconnected forts see fewer; no lasting identities; their stock depends on where they've been: "if you have a trading post, they stop". [early] (conv:101)

### Places, roads and civilization
- Civilization generation makes towns, roads, territories, structures and bad guys, with ground edits (road strips, flat building pads); people and hostile NPCs tuned in a civilization lab. [decided] (WG:571, 592-594; decisions.md:84-88)
- Places in the world: villages, towns, hostile sites, player settlements, ruins, the Temple of Darkness; on the move: caravans, couriers, travellers, wanderers, raiders. [decided] (simulation.md:45)
- Roads made in the civilization stage: a smoothed line laying different kinds of road with transitions between them. [wanted] (decisions.md:731-734; conv:24)
- Generated roads as NPC flow maps, with pathfinding pre-solved. [idea] (simulation.md:412)

### History, news and events
- 1,000 years of history before play: factions build towns and roads, form larger factions and fight; villages of all ages at the start. [decided] (WG:2148-2150; simulation.md:98; conv:545)
- One chronicle service records events; flagged events reach the player's event log, couriers and NPC conversation. [decided] (WG:2137-2144)
- History replayable from a world seed plus a history seed; David means to run one world 300 times, and 300 worlds. [wanted] (handoff.md:290-292; simulation.md:248)
- News particles: no knowledge of distant places until it's carried; distant events arrive by messengers, couriers and word of mouth; dialogue filled from live events, nothing made up. [idea] (simulation.md:81, 286, 297; vision.md:114-115)
- Villages have problems (nearby evil, events, economy), each a potential procedural quest that earns reputation and helps the village resist evil (rats near a swamp). [idea] (vision.md:656-659; conv:28)
- History nowhere near as deep as Dwarf Fortress's: "some history is ran but not so deep like df". [early] (conv:16)

### Rule and politics
- Reputation per town; buy a deed, live there, run the town: "eventually being able to buy a deed and live there". [early] (conv:28)
- Live in a town and do quests for fun; quests arise by chance; elections won by gathering 60% of the signatures, quest-givers voting for you. [early] (conv:198; from-grok.md:25-27)
- Owning a house in a town you don't control makes you its inhabitant. [early] (conv:539; from-grok.md:28)
- Rise to king of a 10-town kingdom, with forts: "appointed king of a kingdom and control 10 towns and build forts too". [early] (conv:26; from-grok.md:24)

## 9 Combat, creatures and danger

### The character of danger
- Lethal and committed: everyone deals and takes a lot of damage; drawing a weapon is deliberate; a single wound is really bad: "combat is highly lethal, a single wound is really bad". [idea] (vision.md:490-491; conv:26)
- No level scaling: a weak player can meet very strong enemies (weak and strong zombies); the unexplored world gets harder by itself: "there is no level scaling". [idea] (vision.md:131-133, 537-538; conv:26)
- Horror by behaviour: goblins two feet tall, weak but cunning and cruel, creeping into houses and knifing people at an inn at night: "2 foot tall monsters creeping into peoples houses and stabbing them". [idea] (vision.md:533-536; conv:472)
- Staring too long at the wrong person can turn them hostile. [idea] (vision.md:203-204)
- Deep in lands of darkness, allies may rob you and flee. [idea] (vision.md:982-983)
- Compelling combat: you have to fight sometimes, with chances whenever you want. [early] (conv:16, 177)

### Fighting
- Fights are local; generic resolution in the Intermediator; a d20 in the World sim; off-screen results through Fate. [decided] (simulation.md:22, 76)
- Combat is the demo's headline: "it has to look so fun everyone tries the demo". [wanted] (watch-list.md:29; conv:180)
- The feel: tricky, but fun and easy once you get it. [wanted] (vision.md:517)
- Pull and release: left click captures the mouse; the attack depends on the quadrant you pull to and how far; David: "the better mechanic for attacks". [idea] (vision.md:506-509)
- You hit if your aim predicted the enemy's movement; a slight quick movement toward the attack helps. [idea] (vision.md:510-512)
- Hold too long and it's a feint: it fades, costs a little stamina and returns you to en garde, with vulnerability windows. [idea] (vision.md:513-516)
- Blocking on timing: pressing at the right moment is one block, releasing at the right moment another. [idea] (vision.md:522-523)
- Arm animation perhaps from David's own living-room motion capture (live webcams into Blender, later). [idea] (vision.md:524-525; mem/mocap-side-project:11)
- Hiding behind corners as an enemy approaches. [idea] (proposal:853)
- Ranged combat and the details of blocking and defence. [open] (vision.md:997)
- Footwork: lots of WASD tapping and sliding; position matters, no stances; drawing a weapon gives momentum and a slide, the camera lagging and smoothing; sweeps are hard in a 1-block tunnel: "combat movement should involve a lot of tapping wasd, and you slide a bit". [early] (conv:177-178; from-grok.md:54-55)
- Feels like a rhythm game and looks cool. [early] (conv:26)
- Three tiers of combos, each harder to pull off and to parry, with straight-line guards, V-shaped moves and ripostes. [early] (conv:32-34; from-grok.md:46-55)
- Weapons have their own moves; skill unlocks better ones and makes them easier. [early] (conv:32)
- Impaling: impaled foes don't advance; pulling back staggers them, stepping in hurts them badly; stronger, heavier foes can shove you off. [early] (conv:178; from-grok.md:46-55)
- NPCs fight with the same animations but do something simpler; NPC-against-NPC fights resolve simply. [early] (conv:32-34; from-grok.md:46-55)

### Battles and war
- Battles epic but intimate (Gilgamesh, the Iliad): about 600 figures in a huge one (100 soldiers against a city of 400 people and 100 soldiers), a great effort for a seasoned player; at most about 800 NPCs on screen: "not trying to make Total Annihilation". [decided] (decisions.md:1562-1567; simulation.md:223-229)
- Boss fights while directing two generals of spearmen. [wanted] (decisions.md:1523-1525)
- Villages raise guards and send them to raid hostile sites; the results are calculated, not played. [idea] (vision.md:662-665)
- Squads equipped and trained Dwarf Fortress style, with simple orders (patrol, guard, deal with criminals). [early] (conv:26; from-grok.md:37-42)
- Armies: squads of 5 under a captain, brigades of 20 under a lieutenant, armies of up to 3 brigades under a general. [early] (conv:196; from-grok.md:34-36)
- Thin-sim war like Risk; raiders equipped from their village's means (smith level, local metal), arriving at your border: "the five elf fighters have copper ringmail and copper spears". [early] (conv:545; from-grok.md:37-42)
- A skilled high-level player with magic gear can solo a 20-man squad or take a fortress. [early] (conv:26; from-grok.md:40-41)

### Creatures
- Creature categories: livestock, wild animals, giant animals, creatures, human foes, humanoids, goblins, plant enemies, undead, supernatural, extraplanar. [decided] (vision.md:636-637)
- Giant animals are beasts, not evil: a rare likelihood wherever their animal lives (a giant elk wherever elk live), most likely in biome cores; they exist because animals have become corrupted; now and then one kills a villager: a hazard like weather, not a faction. [decided] (WG:1486-1498)
- High-level evil druids and rogues can bring beasts; evil factions raise beast likelihood round them or keep their own; a wild one is a "beast" (elk beast), one from an evil faction a "vile beast" (vile elk beast), with its own behaviour. [decided] (WG:1493-1505)
- Evil druid cults are a faction that may lead beasts and giant insects as they grow. [decided] (WG:1461, 1494-1495)
- Giant ants and scorpions in the desert, perhaps led by evil druid cults. [wanted] (WG:1459-1461)
- Taming a beast, with great effort. [idea] (WG:1501)
- Early threats: goblin bands and wolf packs; a haunted cemetery of 20 enemies is a big solo challenge. [idea] (simulation.md:220-222)
- Underground: magic braziers (like Doom II's), cult lairs, spider nests, goblin hovels, trolls, ogres, skeletons and zombies, glowing lichen: "underground dungeons could have magic [braziers] like from doom II". [idea] (WG:1318-1320)
- Any cave, at any time, could hold a really nasty kind of zombie. [idea] (vision.md:134)
- A risk of magma flooding in the ultra-deep. [idea] (WG:1423)
- Dragons in the sky. [idea] (proposal:341)
- Cave monsters from the start that respawn slowly, so a cleared cave stays clear a good while: "they only respawn slowly so you can clear a cave for a good bit". [early] (conv:16; from-grok.md:76)

### Hostile places
- Hostile sites: goblin den, bandit camp, evil cemetery, witch house, mad scientist, troll cave, cult hideout, pirate lair, plus the unique Temple of Darkness and its dungeon. [decided] (vision.md:634-635; simulation.md:47, 333)
- Goblin dens with jobs that are lists of heinous crimes; dens send out raids and harass travellers. [decided] (jobs.md:79-80; conv:16)
- Evil against criminal (as in Daggerfall): bandit camps and pirate lairs are criminal; cults, evil cemeteries and witch houses evil. [idea] (simulation.md:332)
- Savage factions as spawners: skeletons and zombies from graveyards and crypts; necromancers in towers spreading miasma, more common in evil areas. [early] (conv:16, 51, 196; from-grok.md:74-75)

### Evil grows
- Evil grows on its own; each hostile site type has its own growth mechanics, and savage factions advance in the lite sim, so difficulty rises naturally: "adding natural difficulty scaling". [idea] (vision.md:116-118; conv:196)
- A goblin den at 1 hour might hold 3 goblins and a shaman; at 80 hours, 20 goblins, shamans, a chieftain and an ogre; the bigger the den, the higher-level its goblins. [idea] (vision.md:131-133; from-grok.md:77; conv:196)
- Ignored, attacks become urgent only after longer than most people play a Minecraft world; you can still keep a farmstead while the cities burn. [idea] (vision.md:118-119)

## 10 Magic, lore, fate and religion

### The gods
- Two goddesses and one god who are not hostile, and Darkness: Fate married to Darkness, Life-and-Death married to Fire; Fire fathered three children with each goddess, a dwarf, a human and an elf from each: "theres two goddesses and one god that are not inimical". [idea] (vision.md:40-43; decisions.md:398-400)
- Fire lives at the planet's centre; both goddesses inside the moons (there are two); Darkness inside a wandering "planet X": "moon stuff is gonna be big". [idea] (vision.md:44-46; decisions.md:398-400)
- Archwizards theorising a fourth force woke Darkness, who swore to murder all the offspring. [idea] (vision.md:47-49)
- A small, very lopsided pantheon, worshipped for boons, with The Golden Bough as a reference: "i want this pantheon to be very asymmetrical". [early] (conv:99)
- (Fate's gender: see To settle.)

### The player's story
- History ends as the player incarnates and the gods awaken; evil wakes and kingdoms go to war: the Age of Strife begins as the player arrives. [idea] (vision.md:51; conv:196)
- The "seventh child of Fire" is never said outright; a mural of the six progenitors lets you work it out. [idea] (vision.md:52-54)
- The God of Fire appears as an irascible deadbeat father (a speaking stone face?); in the end, "that's my boy". [idea] (vision.md:55-58)
- The chosen one without meddling: the tropes are evoked, but the player is never steered; no prophecies unless one is the only way to make the story work. [idea] (vision.md:63-69)
- Agents of Darkness are sent to murder you, as foreshadowing; a boss fight with Darkness, who tells you who you are. [idea] (vision.md:59-62)
- (Who the player's mother is: see To settle.)

### The Age of Strife
- No player setting for the pace of the decline; your actions set it. [decided] (vision.md:138-140)
- The world weakens itself as kingdoms fight; ruined kingdoms are raided more; power concentrates, then even the strongest are overwhelmed; left alone, evil wins. [idea] (vision.md:124-130; handoff.md:372-373)
- Pursuing the main quest makes the Dungeonmaster draw more often. [idea] (vision.md:113)
- The pace of the decline. [open] (vision.md:135-137)

### The endings
- Ending 1, the gods' quests (as in Ultima): one quest per god, then defeat Darkness; evil settlements stop expanding and may wither. [idea] (vision.md:73-75; conv:196)
- Ending 2, main force: conquer or vassalize the nations and rout evil by strength. [idea] (vision.md:76; conv:200)
- Ending 3, the dark path: summon Darkness with the Necronomicon and enslave the world. [idea] (vision.md:77-78; conv:196)
- The Necronomicon is found under the Temple of Darkness (located from clues), or rebuilt from nine scraps found in bookstores and researched by intellect workers. [idea] (vision.md:78-83; conv:196, 200)
- While the dark ritual runs, every NPC faction attacks you and the hostiles go neutral; afterwards you command the hostiles: "all npc factions attack you with everything and hostile factions become neutral". [early] (conv:196, 200)

### Fate and the Dungeonmaster
- The Fate service: one engine under every living-world roll, knowing the roll's kind and the player's wish; the planet's dice (world generation, fixed by the seed) are never bent; architecturally important. [decided] (simulation.md:3; vision.md:144-146; mem/incus-artus-project:13)
- Fate's twists bend by luck and strangeness, with story progression twisting the odds; the impossible stays impossible; every probabilistic event is hooked and categorised. [decided] (simulation.md:112; vision.md:147-151)
- Players are never told Fate exists; Fate is saved with the world; the dice are reported honestly in the news. [decided] (simulation.md:120-123; vision.md:147-151)
- Chance events are rolled through Fate; story advancement is handled exactly across the simulation levels. [decided] (jobs.md:62-69)
- Astrology as a reason when none is needed (why two people don't get on). [decided] (jobs.md:50-51)
- The Dungeonmaster draws event cards: procedural, yet forming consistent beats. [idea] (vision.md:111-112)
- Fate as the source of chance, luck and malice, scattering "fatedness" that drives people and settlements to less necessary acts such as war: "a god of fate that is part of the larger dungeonmaster". [early] (conv:49)

### Magic
- Magic effects made in the FX Lab (lightning among them); spells throw or cause sprites. [decided] (decisions.md:511-512, 689-692; proposal:91)
- Spells use blue energy, which barely regenerates except in sleep; only dedicated mages rely on magic. [idea] (vision.md:544-545)
- Magic costs energy as a drain (on use) or a reserve (held back while worn). [idea] (vision.md:586-587)
- A few memorised spells, cast from a tab in the personal screen; the equip slot can hold spells. [idea] (vision.md:552-553; decisions.md:493-495)
- Scrolls: usable by anyone, rare and valuable; they load a one-time spell into a slot or act when read. [idea] (vision.md:548-549)
- Tomes: about 6 spell pages, and adding one is very expensive; tomes are reusable scrolls; casting from a tome is slower than from memory. [idea] (vision.md:550-551, 588)
- The magic kit: potions, flasks (thrown potions), spells, rings, amulets, cloaks, scrolls, wands, staffs, tomes, enchantments; content lists in his vault (scrolls, potions, rings, spells, wands of every wood and metal). [idea] (vision.md:554-555, 579-587)
- Wands and staffs hold spells; the base item has its own modifiers. [idea] (vision.md:561-566)
- Fennling, a skill: "fennling is the skill of transferring charge between wands. saves you from carrying multiple." [idea] (vision.md:567-569)
- Spellmaking and enchanting with unpredictable results, "a bit like morrwind or daggerfall or noita". [idea] (vision.md:559-564)
- "Spell lego" as in Noita: bizarre, useless and dangerous spells that are fun to make; remind David when magic design starts. [idea] (vision.md:556-558; watch-list.md:399-403)
- Weapons and armour take magic only through enchanted gems, which can also be set into jewellery or encrusted on furniture and tools: "gems can be enchanted and encrusted onto weapons and armor". [idea] (vision.md:589, 870-871)
- Inherently magical arms and armour are rare: NPC masterworks, quest or structure loot David places, perhaps astrological conditions for enchanting players. [idea] (vision.md:591-594)
- Metals tied to a magical or alchemical system. [idea] (decisions.md:311-314)
- Fasting strengthens spells or energy, but makes you far likelier to die in a fight and is crippling without very high skill: "fasting to cast harder spells is a glass cannon". [idea] (vision.md:372-378)
- Blessings, shape to come. [open] (vision.md:595)

### Religion
- How shrines and offerings work. [open] (vision.md:998)
- Shrines and temples; offerings and ceremonies in a town sway your luck. [early] (conv:49)

### Names and echoes
- Lore names on file: Lacrimae Mundi ("the Tears of the World") and Malleus Verbi ("the Hammer of the Word"); what each names is still David's to say. [idea] (vision.md:85-88)
- Excalibur and Blake echoes, saved at his request, not decisions: the player as Arthur; the land and the king as one, a ruler's state showing in the land; the Dragon and the word; magic passing as the age turns; Blake's forge. [idea] (vision.md:89-107)

## 11 Time and seasons

### The clock
- A fixed-tick simulation, 30 ticks a second (tunable), heavy systems every Nth tick; the Intermediator hourly and the World daily. [decided] (decisions.md:389, 422-423; simulation.md:20)
- One clock that runs on its own from real time, never tied to frames, and can pause or speed up; keep the existing clock: "the timeclock we have works good. we dont want to lose that." [decided] (decisions.md:1033-1038, 1545-1547; conv:424, 441)
- Pause must be possible (in development, from the manager screen, as an optional setting); a paused world is truly frozen; planning works while paused. [decided] (decisions.md:390-391, 820-824; vision.md:188)
- 4× maximum speed is a dev tool that may not ship. [decided] (decisions.md:390-391; simulation.md:93)
- Time frozen while the game is in the background is left out of the world clock. [decided] (regroup.md:246)
- The clock pauses in menus or the manager and speeds up in strategic views. [early] (conv:441)
- You sleep automatically in your quarters with the manager or orthographic view open, if nobody is panicked, and time advances. [early] (conv:322; from-grok.md:108-109)
- (Day length: see To settle.)

### Day, night and moons
- Real night with day, dusk and two moons; very dark nights. [decided] (caves.md:481, 824; decisions.md:393-397)
- The sun moves slowly, and terrain light updates slowly with it. [decided] (proposal:755)
- Moonlight from two moons is "gonna be big"; perhaps astrological conditions for enchanting. [idea] (decisions.md:398-400; vision.md:45, 594)

### Seasons and growth
- Plants change over time and crops grow. [decided] (decisions.md:681; conv:192)
- Seasonal work on areas; irregular winters; emergent temperature (later). [wanted] (decisions.md:348-349, 356; vision.md:638)
- (Seasons and the calendar: see To settle. Food spoilage timings: section 6. History before play: section 8.)

## 12 Multiplayer and network

- Multiplayer is not a goal; later maybe LAN or host-a-friend scale; a basic hookup may come for testing. [decided] (vision.md:906; decisions.md:384-385)
- Every player action is a request with a player ID, so networking can be added without rewrites. [decided] (decisions.md:381-382, 825-826)
- Only the outcomes of fate and chaos cross the network; everything else is derived locally: "only the actual outcome of fate and chaos". [decided] (decisions.md:1466-1470)
- As late as possible: an alpha-tester note box posting to a Discord webhook, then two-way chat, then a developer "drop-in" that doubles as host-a-friend: "a network pipe so that i can watch alphatesters, chat with them". [wanted] (decisions.md:377-380; conv:478; from-grok.md:106-107)
- Hosting as a selling point: never an MMO, never servers; a player hosts their own world and invites guests, at the scale of a D&D table: "like a dnd friendgroup". [idea] (vision.md:907-911; conv:478)
- The taste of StarCraft 1: honest old software. [idea] (vision.md:910-911)
- Co-op, not multiplayer: the host is always party leader and controls time (sleep, pause, speed), the manager and the settlements. [idea] (vision.md:912-914; handoff.md:340)
- Guests have no presence in the story: they are inhabitants under the host and can have their own houses; they can do a quest's tasks, but the host completes it. [idea] (vision.md:915-919)
- A co-op guest is a follower; offline, their character carries on as a person the host keeps in the party or sends home. [idea] (vision.md:244-246; handoff.md:336-342)

## 13 The editor, modding and data

### Data and modding
- Data-driven and moddable: TOML, one file per block, readable folders like X-COM, text IDs (incus:granite), content packs as folders; items and creatures live in data: "should live in data, not be hard coded". [decided] (decisions.md:788-797; vision.md:18, 923; conv:444)
- Modding is a long-term goal: mods add their own pack folders. [decided] (decisions.md:791)
- Live reload of data files, textures and models: changes on disk show at once. [decided] (decisions.md:790, 802; proposal:104)
- Mistakes in data files never crash the game. [decided] (decisions.md:799-801)
- Models from Blockbench now and a real 3D modeller later, via glTF, made by David. [decided] (decisions.md:793-794; regroup.md:1227)
- Art/ is David's messy workshop; assets/ is the delivery folder the game reads. [decided] (decisions.md:547-548)
- A "write this tweak into the block's file" button, later. [wanted] (decisions.md:803)
- Village and site types as data files. [open] (simulation.md:458)

### Saves and versions
- No save or load game: Minecraft-style world folders, one file per system; only changed chunks saved; settings and presets readable text: "it will be like minecraft with worlds". [decided] (decisions.md:807-812; simulation.md:125; conv:446)
- Versions are fine and worlds needn't carry across updates; in pre-alpha a new build means new worlds (old ones load with a warning); modders changing the recipe take that on: "if you are modding with the editor you better know what youre doing". [decided] (WG:1406-1412; caves.md:554)
- Old saves and old presets don't matter for now; only the startup default must load: "old presets don't matter that much to me". [decided] (watch-list.md:439; mem/old-presets-low-priority:11; msgs:213)
- Region files; autosave later. [idea] (simulation.md:148)

### The Editor Suite
- The editor is David's and isn't shipped for now; it may come later as a paid release or bundle; anyone who has it before then is someone David is showing it to: "until the best use of our time is serving eager modders". [decided] (decisions.md:1002-1005; regroup.md:39)
- We're building the suite the world generator is made with, not just the generator; the suite is also an art tool (its stone panels shaded with the world engine). [decided] (WG:1930-1931, 2057-2065)
- The Suite's tabs: Registrar, Graphics Kit, Tectonic Sim, World Compiler, then Civilization; on its own screen (F2), with the physical world paused. [decided] (WG:113-115, 1933-1934; msgs:84, 194-202)
- Ask David before naming or placing any tab, panel or stage; each word means one thing: "the language we use to describe the sections of the machine". [decided] (toy-factory.md:5-6; WG:595; mem/editor-suite-naming:13-15)
- The editor may get messy and can be started over later; tabs by stage, repackaged much later: "would wind up in a psych ward". [decided] (WG:26-27, 601-606)
- Made again, its windows arranged and shaped on purpose, each setting and control placed with David first; no line in a shared top bar that a window doesn't need, none left after what it controlled is gone: "organizing the UI is super important to me". [decided] (decisions.md:1079-1091; mem/editor-suite-naming)
- The new editor's pages reached by a drop-down menu at the top: "software map, tectonics, world gen, registrar, etc". Spelt Software map, Tectonic Sim, World Gen and Registrar: "we should probably call them Tectonic Sim, World Gen, and Registrar". [decided] (decisions.md:1095-1114, 1118-1126)
- The software map (the control room, live) is a page of the editor, not of the game's own panel: "just be a black screen with magenta boxes that show a tooltip when you hover, connected by lines, and inside of larger boxes for sections if need be". [decided] (decisions.md:1068-1070, 1079-1091, 1095-1114)
- Note buttons on each section, saved to David's notes file. [decided] (handoff.md:276-278)
- Eventually making a world becomes a fun, attractive process for players; placeholder graphics until then. [wanted] (WG:1935-1936)
- A world map first: a literal view of the generated world for David that becomes the create-world map; three maps in all (the designer's suite, the create-world map, the in-game literal map with overlays). [idea] (vision.md:671-695)

### World generation tools
- World generation in its own suite with live sliders; three speeds: live, a fast map-scale make, and a full make; a full world-gen stack with deep control panels, tuned before shipping. [decided] (decisions.md:418-419, 956-957; WG:683-690; vision.md:640)
- World Compiler panels: Landmass, Topography, Climate, Mineralogy, Ecology, then the World Interpreter at the end of the chain; Topography in David's order (Land and sea, Ridgelines, Mountains, Hills, Torn coast); Climate holds Rivers, Temperature, Wind and Rain; Mineralogy and Caves have panels of their own; Ecology and Civilization come before the Interpreter: "minerology is gonna need a lot of space". [decided] (WG:370-372, 591-595, 620-669; terrain-pipeline.md:48-55; caves.md:338)
- In the new editor, the compiler and interpreter go inside the World Gen page, "if it even needs an explicit window", and the stack is laid out afresh: "we can definitely organize the world gen stack better this time". [decided] (decisions.md:1118-1126)
- Civilization gets its own page: a library of David's finished worlds to run civilization on. [decided] (WG:603-606, 626-627; watch-list.md:446-448)
- The Tectonic Sim runs live with a speed control and bakes to "world bakes" (maps plus seed and settings); once tuned it goes in the closet: "i absolutely want to watch it move live". [decided] (msgs:12, 84, 87; WG:325-334)
- Render World and Spawn: judge a world by flying over it in person; a minimap spawn picker; observer flight from a camera page, up to thousands of blocks a second, keeping collision. [decided] (WG:1762-1764, 1785-1796; msgs:191, 210, 228)
- A true-scale terrain lab: the close-up at 1×, at any zoom, is the whole view (the map view retired); real surface blocks lit with the game's own lighting, with an off switch; no "because" readout, the hover readout is enough. [decided] (WG:435-465)
- A processing meter whenever the computer is thinking. [decided] (WG:555-557)
- Layers mix numbers, not colours: "ultimately this software is translating numbers to colors and colors to numbers"; what's watched is chosen apart from the layer being edited; layers can be duplicated; image layers as stamps of David's textures (a burlap weave under an ancient temple). [decided] (WG:1999-2041; placer.md:103)
- Move a slider and see the world change, fast while dialling in a small area. [wanted] (WG:456-458)
- Respawn at exactly the same spot and view, for A/B screenshots, and a quick "respawn here with changes": "render world and spawn at the exact same point as before". [wanted] (WG:1335-1339, 2416-2418; caves.md:24; mem/reminder-respawn-same-point:11)
- A "stop after" control over how far the pipeline runs when a world is made, before the polish pass; remind David. [wanted] (watch-list.md:450-457; WG:2392-2403)
- Spline curve editors for the land and sea curves and mountain profiles, adding dots and flexing the line: "i need a curve graph for the land and sea curve… a spline generator". [wanted] (terrain-controls.md:34-38, 54-57; msgs:113, 119)
- Tell David where every implicit noise function lives in the pipeline. [wanted] (terrain-controls.md:79-80; msgs:137)
- Adjustment ("mutation") layers next; a sequential "world type" file built from layers, groups, spline maps and signal routing. [idea] (WG:2006-2009)
- Presets for every panel, an A/B switch against defaults, ez-bake (Space keeps the seed, Shift+Space rolls a new one), scroll-to-change values: "i will require an 'ez-bake' button in the world compiler". [early] (msgs:72, 125, 147, 165)

### The placer
- The placer is infrastructure with no panel of its own, seen inside the systems that use it, with options shown: "its pretty infra but id like to see some options". [decided] (WG:2232-2238; placer.md:14)
- Very powerful and flexible: library blocks, distributors and layers from many fields (tectonic fields, things already placed, whole sections, fields from points such as river ends and peaks), at world creation and any time; nothing hard-coded; shared by mineralogy, trees and ecology: "it cant be hard coded, needs a wrapper to register distributions". [decided] (WG:2215-2252; placer.md:14-20, 353-356)
- Stacked number layers with a chooser (first fit, best fit, chance); column profiles first; fields combine with blend modes. [decided] (placer.md:324-344)
- The field view: a black screen showing the field being made; a depth sweep through the Z levels to see minerals, rocks, ores, soils and sands; a "why is this sand?" readout; placer settings in presets with import and export: "sweep like lidar and see where the minerals, rocks, and ores are going". [decided] (WG:2253-2258; placer.md:330-341)

### The Registrar and colour tools
- The Registrar: David registers and changes blocks himself; laid out as library by tag, dock, master palette; the first goal is to see the blocks and their unsorted colours: "sneak up on the final configuration." [decided] (WG:29-32, 113-122; mem/editor-suite-naming:15)
- Blocks grouped by tags (nested with "/"), not folders. [decided] (WG:116-118)
- New materials by moving a block onto another colour spline (limonite to hematite); a block can draw on several splines; a duplicate is simply a new block. [decided] (WG:100-105)
- Registrar tabs: Blocks, Objects, Blueprints; size settings per object in the Objects tab; flame strips marked by magenta dots, timing in ms per frame, an anchor dropdown, ½ and 2× size buttons. [decided] (decisions.md:660-669; objects.md:192-193, 276-295)
- See-through grid and outline blocks for defining 3D regions. [decided] (decisions.md:566-571)
- Start the palette fresh on HSL splines, HSL staying the working colour language; reshading through the Graphics Kit's palette tools with sliders, as shared code; art always has a way in through the Color Lab; the colour crusher; the Lab's sections fold away: "move everything into a spline as soon as possible". [decided] (WG:85-140, 167-169)
- Claude points out factual problems in delivered art (seams, sizes, readability). [decided] (decisions.md:557)
- An isometric mini preview of blueprints in the dock: "i think an isometric view would be preferred". [wanted] (decisions.md:660-664; objects.md:190-199)
- A blueprint factory (an asset factory world): build with the library's blocks, add hook blocks, register entity assets, with a Blueprints tab, after the caves; its place in the suite is David's call. [wanted] (decisions.md:648-651; handoff.md:220-222; caves.md:3)
- The Color Lab: a global palette added up from all blocks; highlighting; grading against nearest HSL neighbours; effect and UI colours drawn from it; selections, group palettes, export, replacing a colour across assets; named splines; inserting a shade never shifts the others; each asset regulated or not; fast round trips with Aseprite. [idea] (WG:56-84, 106-111, 154-164)

### Graphics Kit and FX Lab
- The Graphics Kit stays as the art pipeline, with oscillators and a loop recorder (record N frames, check the loop is seamless). [decided] (WG:12-15; toy-factory.md:131)
- Retrofit the Graphics Kit rather than build a second container: a modular synth of three devices (Graphics Kit, Sequencer, FX Lab), each its own page in one window. [decided] (toy-factory.md:10-18, 32)
- The name FX Lab: "toy factory is whimsical but we'll probably go with FX Lab". [decided] (toy-factory.md:71-73)
- Palette grading and pixelizing with real control, which David found nowhere else. [wanted] (toy-factory.md:103-105)
- Not a drawing program (Aseprite does drawing): an image editor's adjustments where every slider moves by itself: "Many times I have sat there moving a slider back and forth". [idea] (toy-factory.md:97-101)
- The FX Lab and zone designer modelled on the StarCraft editor's triggers, "an intimidating page for designing triggers"; zones extensible without coding through a scripting page with a menu of hooks, much later. [idea] (toy-factory.md:45-47; decisions.md:716-728)

### Labs and test worlds
- A Caves panel, with a test cave at the crosshair. [decided] (caves.md:338-340)
- A Render page with live sliders, a speed readout, and the card's own clock ("actually very useful"). [decided] (regroup.md:824, 847; handoff.md:92-93)
- A civilization lab: towns and people as symbols on black, 40 hours in a minute or a whole age in seconds with Skip ahead and graphs, overlays of where people go and which sites settlers weigh; replay by seeds; rerun, compare, add pressure, snapshots; its own page. [wanted] (watch-list.md:157-161; simulation.md:164-256; handoff.md:285-288, 374-375)
- (The test world: see To settle.)

### Controls: David's rules
- Every slider exists so David can set it by hand. [decided] (how-its-made.md:15-16)
- Controls do what they say and nothing relies on accidents; each control refined for what it turned out good at; controls that mean nothing to David removed or repurposed. [decided] (WG:1803-1813, 2379)
- Hidden numbers exposed as controls, each with a plain name, a tooltip and a narrow, useful range. [decided] (terrain-controls.md:65; proposal-peaks.md:112-113; WG:1563)
- Slider ranges tightened round the useful area, the good result in the middle of the bar, every variable visibly working at its default: "encompass more of the useful area more finely". [decided] (terrain-controls.md:152-153; WG:366-367, 1528, 2382-2384; msgs:48, 119)
- Avoid settings that only work at their extremes: "avoiding the necessity of extremes". [decided] (terrain-controls.md:154-155; msgs:119)
- Controls that don't apply grey out; they're never hidden: "greyed out is a better convention since we're folding things now". [decided] (WR:571; rivers.md:181; mem/grey-out-not-hide:13)
- Experiments go in behind a switch or checkbox: "make sure it can be disable with a checkbox". [decided] (WS:422; rivers.md:122)
- Controls ordered broad to fine (continental at the top, local further down). [idea] (terrain-controls.md:126-129; WG:1813-1816)

### Debug and instruments
- A debug panel where every tunable is exposed, with named presets and an "everything" preset; live resources listed; spawn a tree or an item; behaviours tweaked while watching them: "these variables should be added in pages of the debug screen". [decided] (decisions.md:832-838; conv:398, 445)
- Debug pages cost nothing while closed. [decided] (regroup.md:53)
- Urges shown as vertical volume-meter bars with a channel strip, like a mixing desk. [decided] (jobs.md:45-46)
- Build numbers, kept out of sight on the debug panel's Overview page. [decided] (caves.md:553; mem/build-numbers:11-15)
- In development builds, the first-bake sharpening shows raw and live for David: "id like to see the sharpening myself, realtime and raw". [decided] (decisions.md:1165-1168)
- A system inspector if slowdowns come back: "if it can happen to me it can happen to someone else". [early] (msgs:63)

## 14 Text and presentation

### Views
- First-person (party) view: inventory and crafting windows float over the world; click items within reach in the world (an alchemy kit laid out on a table). [decided] (vision.md:157-159; conv:488)
- Two other views (names not final): the manager (full-screen lists and sheets) and an orthographic RTS-style view with overlays for blueprints, zones and routes, available while travelling: "more like an RTS or strategy game mode". [decided] (decisions.md:361-370)
- The orthographic view: region selection, labour orders, height-slice scrolling into buildings, people easy to find with see-through markers through walls and trees. [decided] (vision.md:160-161; decisions.md:371-374; proposal:97; conv:491-492)
- Borderless fullscreen by default, F11 for a window: "yeah, borderless fullscreen is probably the best." [decided] (decisions.md:1062; regroup.md:251)
- Optional Panini projection with a vertical field of view; default 100° with Panini on: "everything looks better at 70 but everyone plays in quake pro". [decided] (decisions.md:564-565, 839; conv:480)
- Area borders on mouse-over, resizable. [wanted] (decisions.md:356)
- A question mark over a villager's head while it's pathfinding: "Very cute, very demure." [idea] (decisions.md:769-770)
- Menus and gamepad. [open] (regroup.md:49; mem/counsel-resume:48)
- The orthographic view entered with Tab; choose how many height levels show; a see-through toggle; a three-click 3D region select for digging, felling and placing; blueprint placement; patrol routes. [early] (conv:46, 97, 248)

### The manager
- A full-screen manager with tabs: Inhabitants, Resources, Buildings, Active Jobs, Assignments, Settlements, World Map: "most of it stands". [decided] (vision.md:162-180; conv:228, 246)
- The word lists under the manager's tabs are associations, not decided contents. [decided] (vision.md:165-166)
- The manager's World Map holds only what you've seen, learned or heard. [idea] (vision.md:675-676)
- A name for the list of people you control (not "villagers", "citizens" or "followers"). [open] (conv:256)
- Character sheets show current tasks, needs and happiness. [early] (conv:248)
- One barter screen at a trade depot, a shopkeeper or a wild trader. [early] (conv:228)

### Talking
- Talking by looking: look someone in the face to start; they stop and greet you with an offer in a floating bubble. [idea] (vision.md:189-192)
- Answer with gestures (nod or shake by moving the mouse); choices sit round the face, picked by mouse direction, or by the number row while looking at a face. [idea] (vision.md:193-199)
- Staring escalates: "Hi!", "...yeees?", "What do you want?"; look away too long and the talk dissipates; walk off and get "... okay, see ya then". [idea] (vision.md:196-202)
- Two tiers: most talk is immersive; important talk is cutscene dialogue that captures the mouse, so the switch signals importance; shopkeepers don't need it, starting a quest does. [idea] (vision.md:205-209)
- The cutscene camera takes the most open spot beside them, the two of you filling about two thirds of the screen. [idea] (vision.md:210-211)
- Dialogue as simple prompts that can trigger anything (barter, quests, reputation, dialogue trees): "can dialogue be simple prompts that can perform any action i wish". [early] (conv:177, 184)

### Words on screen
- Player-facing text plain and understated; David filters any sentiment, and the event carries the feeling: "you let me filter the sentiment". [decided] (mem/plain-game-text:11-21)
- Item cards: the name plus the facts, no flavour line: "just rabbit stew with (rabbit, potato, herbs, salt) somewhere on the items card". [decided] (vision.md:415-418; mem/plain-game-text:11)
- Event log lines such as "Durthon Heffeg gives up trying to [work order]", with details from the Inhabitants tab and a "current activity" column. [decided] (jobs.md:111-113)
- Procedural place names ("Darkenwood", then North, South and East Darkenwood); David adds more naming conventions. [decided] (WG:2111-2113)
- Verbose descriptions, like Dwarf Fortress's. [wanted] (simulation.md:397)
- A yawn explains the nodding-off mouse effect: "a yawn would clear it up for any players who get confused". [wanted] (decisions.md:1366-1368)
- Contextual text: UI words that sometimes appear over things you see, perhaps through a perception skill. [idea] (vision.md:472-473)
- The player's chronicle as a diary linked to dots on the map. [idea] (simulation.md:302)
- Procedural quests. [open] (vision.md:1000-1001)
- Procedural quests made like ad-libs (the Silverdeer Glade murder): "theyll be generated procedurally like ad libs". [early] (conv:539)

### Maps
- The player's map can shade regions by the faction whose influence they're under. [decided] (WG:2114-2115)
- Map key colours (first version): ocean dark blue, rivers light blue, plains light green, forests dark green, mountains grey, roads yellow. [decided] (WG:1937-1947)
- "Discovered": revealed on maps and in the orthographic view; "Seen": fog of war shared by the settlement, fading with no observer (an RTS influence): "Fog of war shared by everyone in the settlement". [wanted] (block-properties.md:31-32; vision.md:638-639)
- The player's map: a pencil-drawn fantasy map, approximate, so you have to remember. [idea] (vision.md:673-674)

### The intro and loading
- The world-creation loading screen shows the world being made, live, with lore and art. [decided] (decisions.md:955-956; WG:679-680; vision.md:645)
- A loading screen for the first bake (section 4). [decided] (decisions.md:1165-1168)
- The world-generation wait is the intro: the world forms before your eyes from a toy Tectonic Sim run in reverse. [idea] (vision.md:699-705)
- Animated slideshow art tells the story and shows what you can do; a fantasy map shown once during the intro. [idea] (vision.md:706-711)
- The create-world map: a circular viewport that rolls like a globe; pixel art with modern effects through the 8-bit dark mode; 17×17 map stamps; made beautiful last. [idea] (vision.md:683-693)
- First dialogue: "dwarf_name: hello!" with "1. hello!" and "2. (leave)". [early] (conv:184)

## 15 Release, business and marketing

### The product
- A commercial Steam release; getting the game done matters more than any one method: "getting the game done is more important". [decided] (decisions.md:968-970, 980-981; vision.md:18)
- All art is David's, drawn by hand; no AI-generated images, ever. [decided] (how-its-made.md:13-14)
- All design and tuning are David's; Claude writes all the code: "i am doing *zero* coding. it will *all* be claude". [decided] (how-its-made.md:15-19; conv:418)
- No decision and no art is ever outsourced. [decided] (how-its-made.md:18-19)
- David draws the UI by hand and supplies sliced assets. [decided] (conv:258-262)
- First-release scope and timeline. [open] (vision.md:1006; conv:509)

### Price, editions and mods
- No paid mods, no store, no licences; mods on the honour system. [decided] (marketing.md:104-106)
- The game at about $20; the Editor Suite at about $200 (price open). [idea] (marketing.md:92-93)
- An "editor edition" for modding, built from the in-game structure editor and sold alongside: "buys the editor edition for modding". [idea] (from-grok.md:104; conv:474)
- The Graphics Kit or Sequencer as its own product: David would have paid $75 for it; he'll hand the tools out or sell them, since keeping them back would be selfish; the world-gen tools are a "good luck" case. [idea] (marketing.md:113-123)

### Marketing
- The marketing command centre targets communities, not individuals; emails go only to public game reviewers, Let's Players and the like: "im not looking for individuals with claude API. just forums and communities". [decided] (marketing.md:220-223)
- Many doors, one house: sell both the toys (instant fun) and the brochure (the freshest forever world); keep the scope deliberately hidden from each audience. [idea] (marketing.md:63-68)
- The descent: a camera flies down from the sky into first-person play with no change of rendering, especially at night: "without changing rendering or anything". [idea] (marketing.md:30-34)
- Many worlds, then one: worlds fading faster and faster until one wins as the camera lands. [idea] (marketing.md:49-50)
- A marketing command centre: a plain whiteboard of communities, an outreach log, buzz watch and listening posts, with the Claude API as "election campaign staff". [idea] (marketing.md:205-243)
- Immediate appeal; reviewers and YouTubers spread the word on its depth, making hours of video about conquering the world; a YouTube short that sells the game: "let people figure out how deep it is on through reviewers". [early] (conv:182, 284, 474; from-grok.md:105)
- A no-expense-spared push to a fun, playable demo, with combat as the hook: "so accessible that everyone tries the demo, to make money". [early] (conv:180, 182; from-grok.md:105)

### After release
- After release a build stays as it is for a while; carrying worlds across updates matters only with a large player base: "if we have millions of players and an update cycle". [decided] (WG:1406-1411)
- Expansion packs: drastically different continents (a more desert one, a tropical one); rebalanced world engines for new kinds of land: "an easy way to add content once a generation of players starts getting bored". [idea] (vision.md:758-771)
- Voyages (new game plus): build or buy and crew the largest ship, cross an ocean-world map, reach a new continent with your people and cargo, and sail back while history runs. [idea] (vision.md:749-757)
- The ship and the ocean are illusions you walk around in; you arrive by skiff: "the ocean isn't real, the ship isn't moving". [idea] (vision.md:764-769)
- Seasickness is welcome: "making players seasick on a boat that isn't moving would make me happy". [idea] (vision.md:775-777)
- A next game, "Incus Artus: The New God", with a local LLM as the Dungeonmaster and professional artists (just thinking): "package a local llm as a dungeonmaster to torment the player". [idea] (marketing.md:178-182)
- This game uses every known trick; an "XL" sequel would need new maths. [idea] (proposal:170)
- Oregon Trail mechanics on the voyage, perhaps, but not the full Mayflower. [open] (vision.md:772-774)

## 16 Principles and rules of making

### The engine's idea
- The Metaphysics Engine: a metaphysical world (the form) projected into a physical one (blocks), every feature built "so that 1 always equals 1": the same question gives the same answer anywhere; cleave to projection and interpretation. [decided] (decisions.md:13-21; projection.md:8-14; mem/projection-principle:11)
- Nothing is truly random: the random parts have barcodes on their noise that run forwards and backwards: "*nothing* in my world is truly random". [decided] (decisions.md:20-21; projection.md:16-18)
- Speed follows from projection: the processor keeps the near world, the card what's seen to the horizon, nothing worked out twice; same seed, same world, near or far. [decided] (decisions.md:39-43; projection.md:40-47)
- The whole in one paragraph: the hologram twinned between processor and card, the screen twinned coarse and fine, the hard work done ahead and derivable in isolation: "the hologram is twinned between the cpu and the gpu". [decided] (decisions.md:1466-1470)
- Projected code and the whole recipe use whole numbers, so processor and card agree exactly; same rules, though worlds may differ in detail: "move to whole numbers. otherwise the hologram breaks." [decided] (decisions.md:120-125, 156-159, 1126-1134; regroup.md:322)

### The pipeline
- A linear pipeline: no later stage feeds an earlier one; stages formulaic, linear, hierarchical and modular, with a full revision pass at the end; each feature asked in order: projected, then planned, then stored as differences: "its important we always think in terms of a linear pipeline". [decided] (decisions.md:33-38, 918-922; projection.md:60; mem/linear-pipeline:11-13)
- Each feature lives at the earliest stage that has what it needs. [decided] (caves.md:338)
- Pay a cost once, in one place, and never move it again. [decided] (WG:581-582)
- The renderer reads the end of the pipeline and never feeds back into world gen; the cleverness sits in one place before it: "concentrate the black magic into an analogy for the interpreter". [decided] (decisions.md:44-54, 73-74)
- Rules that apply everywhere, not per-feature hand settings. [decided] (proposal-peaks.md:76-77; WG:1556-1558)
- Systems are data and registered parts, not hard-coded rules. [decided] (WG:2215-2221; simulation.md:205)
- Deliberate rather than random: caves and rivers are formed, not scattered. [decided] (WG:1268-1270)

### How things are built
- Least change: put the intelligence into the system. [decided] (WS:224-225; rivers.md:255)
- Write too many rules, then consolidate. [decided] (WS:632)
- Cheap methods first, kept open for more sophisticated ones; only first steps that won't cause trouble if minds change. [decided] (WG:1879-1880)
- Every trick in the book, in a codebase that stays understandable: "use every trick in the book and maintain an understandable codebase". [decided] (WG:1792-1793; msgs:216)
- Player and NPC interactions developed side by side, so their balance stays in view: "develop the elements of interaction in parallel for the player and the npc". [decided] (conv:390; from-grok.md:95-96)
- Master craftsmen spend much of their time making and keeping tools: use the tools to make the tools, in recursive, fun loops, with cohesion through David's hand on every step. [decided] (WG:26, 2058-2062)
- One exotic system rather than several overlapping; monolithic designs are suspect: "as misguided as the idea of two renderers". [decided] (proposal:358; decisions.md:1382-1383)
- Rendering split into stages: "lets atomize the megakernal". [decided] (decisions.md:1379-1395; dyno.md:234)
- Working backwards from the result makes the goal clear. [decided] (decisions.md:1395)
- Rendering choices that change how art is made are settled early, with the art rules stated. [decided] (mem/workflow-preferences:18)
- Fix at the root, not with patches: "preferably at the root, rather than as a fix". [early] (msgs:414)
- Get each early stage as right as possible, so later ones are simpler. [early] (msgs:119)
- Accept imperfection: "nothing is perfect, everything is always an adaption". [early] (msgs:429)

### Making the renderer
- From black: measure the bare frame, then the cheapest ray, then add each thing back at its measured price: "project the cheapest ray that lets us see a block in front of us". [decided] (decisions.md:1496-1507; mem/counsel-resume:44)
- Graphics first, the simulation's scheduler once there's something to simulate: "getting the vision in the gpu *the right way* is our mortal threat". [decided] (decisions.md:993-998; regroup.md:34)
- The recipe goes on the card before the first sight. [decided] (decisions.md:1086-1089; regroup.md:134)
- Look first, then the dyno: get it looking right and as fully featured as possible, then inspect how the card is used, then add power one measured step at a time: "i want to get it looking rightish and as fleshed out as possible". [decided] (decisions.md:1206-1221; mem/speed-deep-dive-later:11-19)
- Right and fast together, erring on the side of right: "i'd err on the side of getting it right". [decided] (decisions.md:1607-1623)
- Spend the frame headroom gradually: each graphics step talked through first and measured alone: "how gradually i want to spend that wealth". [decided] (mem/spend-headroom-gradually:11; regroup.md:43)
- Every new card-side system measured from its first build. [decided] (mem/measure-from-the-start:11)
- Start small, keep checking performance, isolate any offender: "we need to isolate the offender and not repeat it". [decided] (decisions.md:999-1001)
- Cost counted in capability, not time; one adaptive solution to fill with stuff: "we arent counting cost in time, only in the finished products capability". [decided] (decisions.md:1130-1134)
- No speed idea held back; every good idea kept on a checklist; nothing off the table; measured improvements kept even when unseen: "we are not holding anything back". [decided] (handoff.md:74-79; dyno.md:4; review:9; far-view.md:1122)
- A failed step is a failure in its context, not forever: record what it lost to, and retry when that changes: "some fails may have been contextual and that context may change". [decided] (decisions.md:1432-1437; mem/resume-2026-10-26-lift:11)
- Do the obvious graphics upgrades now and delay the rest until delaying would cost; make earlier code lighter as each lands; don't fix early what later fixes faster. [decided] (watch-list.md:319-327; far-view.md:814)
- The bar for moving on to content, Counsel's, accepted by David: smooth and cool in his own flight; the land reads as one land; built things hold up at a distance; heavy content doesn't change the answer: "i think you are right with that bar". [decided] (decisions.md:1595-1605; regroup.md:1432)
- Bevy goes, with our own scheduler; each kept piece sits behind an "ejection seat"; Bevy is judged on its method and results together: "we're not having any mercy on bevy." [decided] (decisions.md:973-975, 1022-1030, 1558-1560; regroup.md:82)
- Rip out and rebuild rather than run two engines, as rivers were: "we got rivers working by ripping them out first". [decided] (decisions.md:1483-1494; far-view.md:1404)
- The regroup: a ruthless cut, keeping what David is happy with (the editor, world gen, the simulation, the setup): "as long as we don't lose anything that we are happy with". [decided] (decisions.md:968-972; regroup.md:12-13)
- After that bar, the remaining graphics work is taste and FX. [wanted] (decisions.md:1603-1605)

### Pace and judgement
- Nothing is done, only done enough to go on: "nothing is done, everything is only done enough to do the next thing". [decided] (decisions.md:921-922; terrain-pipeline.md:105-106; WG:2208-2209)
- Nothing temporary: build what the player will see; instruments are fine: "we're now building what the player will see, and nothing is temporary". [decided] (decisions.md:1071-1085; regroup.md:131)
- Don't fix problems before they're seen; deal with them when seen: "i don't think we need to try and fix problems". [decided] (WS:100-101; regroup.md:459)
- Holes get fixed at once; shading and settings flaws are written down for later, as are set-aside problems and oddities: "shading and setting flaws just need to be identified and remembered". [decided] (mem/flaws-identify-and-remember:13; far-view.md:176, 825; watch-list.md:461-462)
- Small fixes come as David points them out. [decided] (WG:2172-2173)
- Refactor religiously: step forward, then retrace and fix what the step affected, reassessing every earlier level: "assess every single previous level of the software again". [decided] (decisions.md:913-917; terrain-controls.md:156-157)
- Judge across many seeds, not one; terrain can't truly be judged until it can be walked with trees and water: "i need to see patterns across seeds, rather than an individual seed". [decided] (WG:1800-1802; terrain-controls.md:156; msgs:137, 207)
- Stay light on our feet: detailed plans near, loose plans far, forks decided by measurements or by David's look: "we have to stay light on our feet". [decided] (decisions.md:1006-1011; regroup.md:121)
- Begin cautiously, knowing something may have been forgotten; step by step, with a launch: "as long as we all understand that we might be forgetting something". [decided] (WG:2250-2252; placer.md:356; rivers.md:812)
- When control is lost, disconnect the parts and rebuild the working ones one at a time; expect to try and scrap several methods: "implement the parts that work more carefully, one at a time". [decided] (WS:28-31; WG:1515)
- Don't change what works; cut what only makes it worse; his drawing wins, the look over the numbers. [decided] (simulation.md:363; rivers.md:210, 752)
- Play with every setting: odd settings that feel neat might "be something". [decided] (decisions.md:842-843)
- Work horizontally (deepen what exists) before adding new kinds of field. [decided] (WG:2053-2055)
- One tool at a time: David tries, Claude gives options, David picks; every element hand-selected. [decided] (WG:1951-1954)
- Vertical progress over horizontal content: "im climbing a ladder not tilling a field. vertical objectives." [early] (conv:42, 173)

### Order of work
- The build order of 2026-10-22: the Fate service, the two-scope simulation, saves, the world-gen suite, world gen tuned as data, then terrain, the first dwarf, and much later combat. [decided] (vision.md:790-798; conv:529)
- Combat built one stage of a swing at a time, after creatures that can attack and block back, well after the first dwarf: "build the combat processor one step at a time". [decided] (vision.md:527-529; watch-list.md:33-34; conv:529)
- Since the regroup (2026-11-05): graphics first; after the bar, the blueprint factory, then the simulation; the behaviour engine the last big technical piece. [decided] (regroup.md:1432; proposal:585; mem/counsel-resume:48)

### Roles and the living truth
- Four roles: David (vision and calls), Builder (builds), Counsel (observes and protects the plan, informs Builder, conceives updates with David), Oracle (questions the structure and the axioms): "ok, so Builder, David, Counsel, Oracle." [decided] (decisions.md:931-937)
- Living truth kept apart from the journals: a live master map (like a power-plant control room), a master plan, and the spec; the plan lives in files, not in a session: "we need a ship list, or the objective spec for the game". [decided] (decisions.md:938-947; msgs:423; mem/team-roles-and-living-docs:11-16)
- A rewrite is being weighed; if it happens, "old Incus Artus" is kept aside as the reference it's checked against. [decided] (decisions.md:948-950)
- A fresh-eyes audit once the dyno settles: "a thorough check we missed nothing and left no performance on the table". [idea] (mem/fresh-eyes-audit-after-dyno:11)
- David wants to follow the shape of the whole system: "follow the shape of the system of systems". [early] (conv:420)

### Working with David
- David designs, makes the art and tunes; Claude writes all the code and explains as it goes, stopping after each step and at every real design choice. [decided] (decisions.md:911-912; conv:418)
- Claude never invents final art; placeholders are black boxes with magenta outlines and bright pink text. [decided] (decisions.md:490; mem/workflow-preferences:14)
- Plain explanations; launch only on his go; ask for assets ahead; one or two test steps at a time; size work by Claude's hours plus his looks, never person-days: "give me 1-2 setting tests at a time". [decided] (mem/workflow-preferences:11-19; mem/one-or-two-steps:13; mem/time-estimates:11)
- David is he/him everywhere: "refer to me as he/him/his etc." [decided] (mem/no-they-them-for-david:11)

## Appendix A: Changed his mind

What was once said, and what replaced it.

### The world
- A circular ocean of 100,000-block radius (Grok era) → a circle about 72,000 blocks across. (conv:10; watch-list.md:529-530)
- A world 400 tall: an "ozone wall" at Z 400, the sky at Z 399, the tallest mountains at Z 333 → build height 1,024, with peaks towering higher; 333 survives as the ridgelines' measuring stick. (conv:59; decisions.md:206, 214; WG:343, 1568-1572; terrain-controls.md:19-20)
- Spawning in the middle of the world (the first walkable test worlds) → see "Where you start" in To settle. (WG:1785)
- The tilt lifting land above the sea on a plate's high side → the edge rise. (WG:410-422)
- Plate cracking → switched off 2026-10-27: "it just doesn't behave or do anything useful after testing". (WG:296-301)
- Foothills → cut 2026-10-27: "if i can't get anything out of it after that, we're slashing foothills". (WG:428; terrain-controls.md:25-27)
- Mountain passes along each ridge (a wobbly crest) → cut; ways through come from continents without long unbroken ranges. (WG:429; terrain-controls.md:69-74; msgs:119)
- Rift valleys → removed, as they "seem to do nothing except make a couple ocean spots a little deeper". (WG:2380-2381; terrain-pipeline.md:40)
- The Lift ("a tablecloth draped over a series of points") → dissolved into the peaks 2026-10-27. (terrain-controls.md:47-50; proposal-peaks.md:17-18; WG:1625-1630)
- The old Interpreter hills ("none of the controls ... do a single thing right") and crags feathered out from the mountains as hills → cut 2026-10-28; ground detail became the Hills. (terrain-pipeline.md:177; WG:1679-1700)
- An Additives tab to become "Mountains", with a Hydrology sibling → the Topography panel. (proposal-peaks.md:19-21; terrain-pipeline.md:42)
- Biomes as areas of influence expressed as cells → biome cores. (WG:2107-2108)
- The Climate order Temperature, Wind, Rain, Rivers (then Mineralogy and Caves inside Climate) → Rivers first; Mineralogy and Caves on panels of their own. (WG:1096, 1239-1240, 2353-2354)
- Springs at slope feet, and snowmelt → cut 2026-10-31: "yes, cut them". (WS:305-311; rivers.md:49; WG:1099)
- Rain-driven rivers, and springs in a height band or from lakes depending on rain → rivers don't come from rain. (WG:774-775; rivers.md:712)
- River ranks by width (spring 1, creek 2 to 4, stream 4 to 8, river 8 to 32, great river 32 to 128) with a bank table, and rank by stream order → sizes by area. (WG:771-777, 820-823)
- Rivers at least 8 wide and great rivers 64 or more → no great river (but see To settle). (WS:607-608)
- Smoothing by lengthening (a squiggling line), meanders, lean and wander, corner banks jutting up on bends, bank fades by bend → the sideways moves and the bends cut 2026-10-31: "cut the bends. cut the sideways moves." (WG:778-779, 928-1012; WS:72-73, 199-224)
- At least half the inland shore flush → shores flush as normal. (WS:814-822)
- Tarns and marshes → "later or never"; swamps wanted instead. (WG:1099-1100)
- The waterfall cave (a river with nowhere to go falling underground) as the first kind of cave → a simple dead-end cave worming into a mountainside; the waterfall cave deferred. (WG:1264-1267; caves.md:335-337)
- A caves-and-cliffs pass after forests → caves next, before trees: "i think that the caves are the scariest thing coming up". (WG:1234, 2335-2338; watch-list.md:385)
- The first test world, a 100-block flat cylinder → the test world with fixed viewpoints (see To settle). (decisions.md:895-903)

### The look
- The whole game drawn in 256 colours → the world in about 600 chosen colours, freely shaded; 256 for the UI only. (conv:302; mem/graphics-goal:11)
- Fog "matters little" → fog wanted at a couple of miles (2026-10-28). (WG:1876-1878)
- Fog as a fixed wall → fog comes with the weather; clear days see the whole world (2026-11-06). (decisions.md:505-507, 1160-1164)
- Light static for now, moving shadows not a priority (2026-10-28); "plain lighting for now, pretty lighting later" → moving torches and light from effects required; blocky per-face light everywhere (2026-11-06 and 11-10). (decisions.md:491, 515)
- Smooth placed light → hard per-block steps, smooth only as an option. (regroup.md:1246)
- A Skyrim-like HUD → the bottom bar with three glass spheres. (conv:177)
- A single isometric view → two views: the manager and an orthographic RTS-style view (2026-11-04). (decisions.md:359-370)
- "Two bands" (his "powerglide": blocks out to pixel size, then one far band) → twin layers. (decisions.md:990-992)
- "Every single thing marched" → the world marched and things drawn; a hybrid renderer allowed (2026-11-05). (handoff.md:171-173; decisions.md:78-83, 979-983)
- The custom renderer parked, then unparked; Bevy rebuilt as a voxel monster → our own renderer, Bevy going. (regroup.md)

### Performance
- 240 to 250 fps on the laptop as a stand-in for "playable on a 980 Ti at 1080p on low"; potato PCs down to a 980 Ti at 1080p on economy settings → a 980 Ti as the price of admission at the standard look, a smooth 60 at 1080p (2026-11-05). (WG:1776-1777; msgs:213; release.md:64-66)
- 250 fps stable with the processor at 90 °C and the card at 85 °C (2026-11-04) → the spec of 2026-11-05. (decisions.md:1496-1507; mem/counsel-resume:44)
- "Unbreakable 120 or smooth 240 on a nice gaming laptop" → the spec of 2026-11-05. (decisions.md:1516-1518)
- "A smooth 60 for anyone, 120 on nice rigs" → the spec of 2026-11-05. (proposal:119)
- A 2011 desktop at 1080p 60 as the low end → withdrawn. (mem/counsel-resume:46, 54)
- A frame cap at every second refresh → a cap just under the screen's top rate. (release.md:29-32)
- A stutter every 2 to 3 seconds at extreme flying speed acceptable for now (2026-10-28) → stutter is a dealbreaker; the frame never waits on the world. (WG:2429-2430)
- A performance overhaul once forests are in, with caves, trees and rivers working first (2026-10-31) → graphics first since the regroup; look first, then the dyno. (WG:2289-2292)
- "Bevy, latest stable, version locked" → Bevy goes, with our own scheduler. (decisions.md:909)

### Player, survival and building
- The same rules for NPCs and the player, pure → the same rules plus a player edge (2026-10-30). (block-properties.md:21; decisions.md:259-260, 275-278)
- Each food's protein, carbs and nutrition with set roles (carbs fast stamina, protein mends blood and injuries, nutrition refills energy daily) → partly replaced by the colour scheme and fat-and-carbs fuel (2026-11-09). (vision.md:274-278)
- Quartile pieces of .25 × .25 × 1 → bars of 1 × ½ × ½. (conv:378)
- Trees falling in a predictable line, tunable live → a slightly random fall, rolled at the cut. (from-grok.md:92-94)
- Registrar tabs for blocks, items, decorations and entities (2026-10-30) → Blocks, Objects, Blueprints. (decisions.md:660-669)

### Combat, time, network, business
- The gestural trace (feet committed, the camera held to about a quarter, a cursor tracing the path, the attack chosen from movement plus mouse path, "billiards"), and the early gestural guards and V-shaped moves → pull and release (2026-10-30) for choosing the attack. (vision.md:492-509; conv:32)
- Four months of 30 days, or 21-day seasons ("that's 2 hours for a season") → see "Seasons and the calendar" in To settle. (conv:42; watch-list.md:531-532)
- Multiplayer "impossible" for a game like this, a stop-time function instead → hosted co-op someday. (conv:228)
- A limited licence to sell mods → no paid mods, honour system (2026-10-26). (marketing.md:92-93, 104-106)

## Appendix B: Suggested, not confirmed

Ideas from Counsel, Builder, Claude or Grok that David hasn't confirmed. One line each.

### World and water
- Counsel: the second "re-read" pass in detail (a coarse scout, small hand-on edits, checking locales against the finished world). (decisions.md:101-115)
- Counsel: the peaks proposal as written (Quilez "More Noise" plan, the original panel layout); the Lift's machinery to scale ridged mountains near points; each mountain seeded from where it stands; a rule for overlaps; Additives whole-map, the Interpreter per-block. (proposal-peaks.md:37-153; WG:1528-1567)
- Claude: World Compiler controls David never asked for (soften coast, hills and detail masks). (terrain-controls.md:10-22, 93-95)
- Claude: a grid shaker and rock-unevener; they did little and were removed. (msgs:246, 414, 423)
- Grok: streaming mountains from extracted blobs as particles (tried as a first experiment), streaming particles from the ridge detector (dropped: "ridges were the wrong approach"), a noise-ridge amplifier. (msgs:251-406; conv:561; WG:510-512)
- Grok: numbers since superseded (a 100,000-block radius, 21- or 30-day seasons, a 218-block list). (watch-list.md:527-533)
- Counsel: rejector details (reject early, hard rejects plus a score, the player picking from the best three, 100 worlds overnight to find the commonest reason). (WG:490-502)
- Counsel and Builder: never treat Z 0 as fixed. (WG:1397-1400)
- Counsel: vertical exaggeration as one scale number; sight-line sums from 400 blocks up. (WG:2460-2470)
- Counsel and Builder: rivers ranked by flow; rain setting each spring's share; salt lakes and dry flats; a menu of tarns, oxbows, floodplains and marshes. (WG:743-760, 1076-1091, 1153-1157)
- Counsel: wind from blurred temperature turned sideways, with curl noise; a fluid simulation later; rain worked out by looking upwind (built, never explicitly confirmed); the lake effect by two passes, or basins first. (WG:1112-1170)
- Counsel: climate fixed through the 1,000 years of history; every field with an overlay and a "because" readout (David declined it in the lab); four seasons with an average and a seasonal swing per place. (WG:1205, 1228-1230)
- Counsel: deserts from rain shadows; a desert stamp on the peaks' pattern; scrub to dry grass to sand; deserts kept apart and off the coast. (WG:1467-1482)
- Counsel: separate Hydrology and Climate tabs (David chose Climate holding the rivers). (WG:607-616)
- Counsel: water cues in detail (a lush band wider for bigger water, reeds, lines of trees along rivers seen from far off, insects, water louder at rapids). (WG:1210-1215)
- Counsel: swamps from a water-table field or from placed ponds, after the rivers and before Ecology. (WG:1377-1389)
- Counsel: "water follows the land at the large scale, the land gives way at the small"; Builder's option B (valleys shaped along rivers, fine layers fading toward the floor); Builder: the land makes way for rivers. (WS:774-807; water-inventory.md:203)
- Counsel: the webbing read as valley lines between rises, also useful for roads and settlements. (WS:874-891)
- Counsel and Builder: the compressor's attack and release along the river; water rising to meet its banks. (WS:843-848; WR:66-83, 274-278)
- Counsel and Builder: picking lakes first (basins ranked by volume, an "always a lake if deeper than" slider, weights for rough ground, stamped lakes). (WS:539-553; WR:153-178, 326-334)
- Counsel and Builder: dry beds wet in spring and dry in summer; petering out after N blocks without joining; "a tributary carries at most" a share of the main branch (shelved); a lake passing its rivers' water to its outlet (added unasked). (WS:444-445, 673-688; WR:221-226, 412-428, 1024-1025)
- Counsel: a dry block between rivers side by side (overruled by David's waterfall merge); four water stages, not five; springs reading steepness; which sliders to judge now; the rebuild process (goals first, stages written before code, one stage at a time beside the old). (WS:262-265, 489-506, 698-709, 952-961)
- Builder: Rosgen river types; flush shores at Z 127; overlay waterfalls. (rivers.md)
- Counsel: water details (still water placed by the generator; spreading only when disturbed and only in the Local sim; natural water as endless sources; the lake-rim fix; renderer options for water's look). (decisions.md:466-483)
- Questions Counsel raised that David hasn't answered: the sea coast and beaches, the shore at Z 127, the sand flats, marshes, how big the biggest rivers and lakes should be, whether rivers ever split going downstream, how long making the water may take. (WS:561-585, 747-751)
- Counsel: caves planned then carved, as kits, beyond what David took: cave kinds (limestone levels, lava tubes, sea caves, fault caves, mines); a grammar of pieces plus rare set pieces; density set by rock. (WG:1274-1303)
- Builder: more cave kinds (limestone, lava tubes, sea caves, halls, geodes); a waterfall cave kit; rock skin. (caves.md; handoff.md:217-218, 228)
- Counsel: a reason for each underground dweller (braziers burning without fuel, cult lairs near the Temple of Darkness, spiders in damp limestone, goblin hovels near mouths, trolls under mountains and bridges, undead where people died, lichen marking old water routes); a lair's fire at a mountain cave mouth visible for miles at night. (WG:1308-1334)
- Counsel: one far-view technique for everything that breaks the surface; the "holes in the canvas" as a layered heightmap; a cave probe to try both. (WG:1346-1366)
- Open for David from the caves work: what "stuck to it" means; how solid (about 0.7) small far cave mouths should be. (caves.md)
- Counsel: three sizes in Mineralogy (rock layers, ore bodies, inclusions); inclusions as the prospector's clue; big mines needing timber, carts and smelters; strip-mine scars; strata following the surface near the top and lying flat deep down; disruptions from the plates; inclusions limited to host rocks; deposits from climate; settle the world's depth first; magma flooding with fluids in the Local sim. (decisions.md:324-344; WG:1434-1450)
- Counsel: the ores-by-depth table and the viability checklist (fuel, copper and tin or iron, flux, salt, flint per region). (ores-by-depth.md:19-78)
- Counsel: ore depletion (ore comes out slowly per block; labour limits a settlement, not the vein; sims mine abstractly; solo players go for rich, deep pockets; slow renewables such as bog iron, river gold and salt pans; cleared rock feeds construction; balance it in a civilization lab). (decisions.md:287-302)
- Counsel: the placer as registered fields plus stacked distributors; placements during play saved as seeded edits. (WG:2222-2272)
- Counsel: a starter block set (dirt, mud, sand, gravel, snowy ground, 3 or 4 rocks on splines, ice, packed snow, wet sand); crossed-sprite grass as a decoration block. (WG:2189-2196)
- Counsel: hunting thinning a per-area number that recovers; pets as person-like records; livestock as animals or as goods. (WG:710-720)
- Counsel: Ecology's plants projected with only changes stored; bulk 2D plants as crossed sprites; plant stages from the calendar; coarse maps for regional change. (decisions.md:677-688)
- Counsel: a moon-phase calendar of nights; each goddess's moon carrying her nature; Fire's glow from the deep; darkness as cheaper light. (decisions.md:401-414)

### Look and renderer
- Counsel: the far view asking "what's here?" coarsely; small openings averaging into darker cells; far lights coming from placed lights. (decisions.md:55-60)
- Counsel: two passes of the same march split at a distance, the far one at quarter resolution; a pixel looks best covering about one store cell. (decisions.md:1095-1101, 1177-1181)
- Counsel: upscaler candidates (xBR or Scale2x edges, Kuwahara painterly, fog in dithered steps, mosaic pixel growth). (decisions.md:529-538)
- Counsel: the palette pipeline's order; mipmaps never blending colours. (watch-list.md:193-207)
- Counsel and Builder: far builds as stand-ins and CDLOD; blocks far out only where something was built. (watch-list.md:240-287)
- Counsel: rock in 3D layers and block light underground; one far-3D contract. (watch-list.md:393-397)
- Counsel: trees (hooks; a tree as one thing; growth swapping stages; a falling tree as a moving voxel object, or its own small marched grid; auto-baked tree sprites; far trees as impostors). (decisions.md:585-598; mem/counsel-resume:34)
- Counsel: one sprite-particle system for flames, leaves, chips, sparks, splashes and spells; spell sprites carrying moving lights. (decisions.md:693-699)
- Builder (objects): flames facing you at full brightness; objects lit flat like blocks; small objects not drawn past the near chunks; instancing; far lights for a lair's fire; picking up and holding objects later. (objects.md:13-19, 57-88)
- Counsel (objects): an ore as its own block drawn as an overlay; natural moss from projection; random sets picked by a position hash with weights. (objects.md:407-429)
- Builder: palette cycles (an index layer, step offsets, wrap or bounce, fading to baked colours far off); permanent spline IDs. (WG:145-153)
- Builder: the graphics-run items (palette-preserving mipmaps, brightness options A, B and C, slot textures, a shared height-tint top, sun and shade colour, a test stand as a stamp). (WG:2161-2170)
- Counsel: "fail like a lens" as a rule (David's dirty-glass words); DXC shipped; the bake hidden in the opening; moving things in small grids and a list of moving lights; clicking far things; a dithered near and far band; palette-slot textures; weighted blended glass; light through see-through blocks; an input-to-photon gate. (regroup.md; dyno.md; proposal)
- Counsel: the Quake look as chunky silhouettes, crisp textures and unblended keyframes. (vision.md:947-951)
- Builder: a Doom sprite flame. (caves.md)
- Claude: a final pass snapping the world to the 256 palette; rejected. (mem/graphics-goal:11)
- data-files.md, built by Claude without a David quote: tags as nested folders; `bar` blocks. (data-files.md:42-45)

### Performance and engine
- Builder's dyno list: rays ending in total fog (David: save for last); relaxed cone stepping; empty-space distance for caves; the sea as an analytic plane; f16; workgroup shape. (decisions.md:1283-1292)
- Counsel's dyno list: light kept in the hologram; timewarp; store layout; blue-noise jitter; ray-tracing cores as a skipper. (decisions.md:1294-1303)
- Builder's routes A to E from the whole-software review (a hybrid near field, light kept in the world, the far march lean and reused, the card kept busy, one source of rules shaped for the card), waiting on David and Counsel. (review; mem/resume-2026-10-26-lift:11)
- Counsel: the "stage" read as overload showing as things arriving late or simpler; separate lanes with budgets. (decisions.md:1552-1556)
- Counsel: the recipe's twins exact, the screen's within tolerance. (decisions.md:1471-1475)
- Counsel: how to apply projection; the processor and card split; temperature as a weak measure. (projection.md:50-78)
- Claude: the old engine's performance plan (uniform sections, greedy meshing, LOD rings). (decisions.md:1629-1641)
- Claude's brief "graphics stay lightweight… quality worlds over quantity", pasted by David but at odds with "sparing no expense" (see To settle). (conv:484)
- release.md plan, no David quote: link-time optimisation; a baseline CPU; compressed textures; warnings-only logs; leaving cores free; resting when minimised; Low, Medium and High presets; logs in the user's data folder; a way to send reports; crash dumps; trimming the world's memory; compressed and background saves; minimum and recommended specs near release. (release.md:14-65)
- release.md, built but not David-confirmed: player copies built without the dev feature; diagnostics (session, crash and freeze logs) on in the player's copy. (release.md:9-13, 41-49)
- Counsel: floating origin; report-file contents and privacy. (watch-list.md:482-519)
- Counsel: the orthographic camera switchable in any world (zoom, pan, 90° turns) with a soak-test leg; the soak test logging processor and card load; the work order of 2026-10-31 (partly overridden by David putting caves first). (WG:2293-2343)
- Counsel: Intermediator and World agreement tests; save compatibility; picking a target machine (500 villagers on screen was Grok's figure). (watch-list.md:71-101)
- Counsel, Builder and Claude: before players keep worlds across updates, version the recipe or save every chunk once made; record the build version in each save and warn on older worlds. (watch-list.md:435-448; WG:1413-1417, 2411-2415)
- Counsel: saves holding several worlds, for voyages. (watch-list.md:166-182)
- Claude: pawn movement "fixed tick, smoothed between ticks" (no answer recorded). (conv:452)
- Claude: save Fate's odds, the Dungeonmaster's deck and evil's growth with the world (Fate's saving has since been decided; the rest has no answer). (conv:519)

### Player, survival and building
- Counsel: the player's edge as one tunable number per activity. (decisions.md:276-278)
- Counsel: more mouse-feel instruments (weight when exhausted, tremor with fear, heavy-weapon inertia), each with a player-adjustable strength. (decisions.md:1360-1371)
- Counsel: fuel shown on the gauges (thin bright carbs, thick golden fat); hover words fat, starch, protein, vigour; a food colour filling its matching sphere (red blood, blue energy, green stamina). (vision.md:289-305)
- Counsel: salt from dry basins as dry regions' trade good. (vision.md:317-319)
- Counsel: year-length arithmetic; winter as the test; an autumn harvest rush; a "sum times a variety bonus" dish formula. (vision.md:359-368)
- Counsel: food names (small game whole on a spit; fish filleted; steaks, chops, jerky; tallow and lard for candles; salt pork and ham; pottage, stew, pie, pasty, sausage); warm colours, steam and sizzle; a herb-cured haunch losing less than a salt-cured one. (vision.md:402-414, 428-429)
- Counsel: the quarter log as the quartile oak's block; fires as gathering zones; the magic loaf as a first lesson. (vision.md:482-486)
- Counsel: inventory as one family per material and form; placing from a mixed slot; the slot showing its mix, such as "12¾". (vision.md:815-821)
- Counsel: the chisel (a clean cut loses nothing; speed by tool, material and skill; villagers may spoil cuts). (vision.md:825-830)
- Counsel: felling extras (branch fallbacks; a warning creak or shadow before a fall; the whole fall worked out at the last cut; crack stages; chips coloured from the texture; drops settling as small objects). (decisions.md:615-647)
- Counsel: workshops in three layers (a signature station per trade, generic furniture for size, tools on the tables for detail). (vision.md:840-851)
- Counsel: system items real and owned while clutter is decor; variety by palette swap; placeable-but-not-pickable decor; "tables, things can sit on it" (never sent to David). (vision.md:860-865; watch-list.md:431-433; mem/counsel-resume:28)
- Counsel: quality in three layers like a wand; an item remembering its maker; gems tying mining to magic. (vision.md:872-882)
- Counsel: zone candidates (pasture, farm field, tree farm, mine or quarry, graveyard); one hook system shared by zones, the FX Lab and spells; triggers read as sentences. (decisions.md:705-729)
- Counsel: the factory-world round trip like Aseprite's; a sun slider; entity palettes from their blocks; room for partial blocks. (decisions.md:745-754)
- Counsel: path-bake exceptions measured per structure; a structure that re-bakes too often goes live-only. (decisions.md:779-782)
- Counsel: show the pull with the cursor; feint timings as sliders; enemies reading and feinting the pull too. (vision.md:518-521)
- Counsel: a gesture reader against a training dummy, built early. (watch-list.md:35-37)

### People, story and magic
- Counsel: talking by looking in detail (at most four choices at a time; how long a look counts; one talker at a time; the quickbar borrowed only while looking at a face, never mid-fight; cutscenes kept rare; a camera fallback for cramped rooms). (vision.md:212-222)
- Counsel: a schoolhouse and teacher; research written into library books that can be copied, traded, stolen or burned; a guild hall serving skill and social needs. (vision.md:254-260, 272-273)
- jobs.md proposals: one "pressure" dial for opportunities, likes, schedules, moods, weirdness and Fate; jobs saying who made them and who can take them; occupation lists as data files; no retrying an order for a while, and reservations; disliked jobs lowering leisure thresholds; the three levels running the same rules per tick or per hour; what gets saved; the build plan. (jobs.md:30-163)
- Counsel: civilization lab extras (branching, compare, graphs, parallel runs; "the lab must run the game's own rules"); rules as a query plus a formula; a city of 500 to 1,000 blocks; news token details; alignment as temperament and crime per settlement; the border as a footprint; bandits made, not spawned; the inn as a doorway; door states; a routes network. (simulation.md; handoff.md:289-293)
- Counsel: the rename "Regenerate the people sim ground" (left to David). (handoff.md:159-160)
- Counsel: the Age of Strife (the world can be lost but not the player; a slow tide with Fate evening the pace; brakes such as refugees and alliances; players must feel the decline). (watch-list.md:138-164)
- Counsel: wand frames from wood or metal; made spells rolling once; spell lego sharing the FX Lab's pieces; a worn item's energy reserve shown as locked fluid in the blue sphere. (vision.md:570-578, 596-598)
- Counsel: Malleus pairs with Incus (hammer and anvil); the Excalibur and Blake readings themselves. (vision.md:87-107)
- Counsel: Dungeonmaster event cards as data with clear inputs and outputs (for The New God). (marketing.md:184-190)
- Counsel: a misty dawn that hides unbaked land, the fog's reach following the bake (David accepted "stage magic" weather in principle). (vision.md:717-727)
- Counsel: voyages in detail (the crossing covers world generation; new continents as data; rocking that can be turned down; an arrival spot on the shore; voyage events from Fate). (vision.md:760-788)
- Claude: the "Goddess of Fate" (see To settle). (conv:516)
- Claude: open questions on permadeath, playing on as a follower, economy and taxes, magic schools, animals and mounts, character creation, audio and the release plan (reported answered in vision.md, not answered in these files). (conv:496-509)

### Multiplayer, tools and business
- Counsel: co-op technicals (Steam networking, Fate bending for the host, guest prediction, player IDs); co-op's own pause rule. (watch-list.md:209-238; decisions.md:1060-1061)
- Counsel: blocks as data files; stable string IDs; a preview lit like the world; shared textures with tints; registered textures copied into tracked assets; reload without restarting. (WG:33-47)
- Counsel: splines as the recipe and the palette as its bake, each slot's origin recorded; one palette file written only by the Color Lab; slot textures; two forms of "replace"; one shared set of palette tools; colour counts and near-twin flags; "new colour" meaning an exact match; every slot colour unique; backup or undo when the Lab rewrites PNGs. (WG:127-131, 172-199)
- Counsel: the FX Lab's design (stable setting addresses, base value plus movement, wiring as scripting, 2D first, baked against live toys, the peripheral tool list, generators for bolts, cracks, particles and pixel sims, output snapped to the palette, poking, GIF export, a cost meter, a build order). (toy-factory.md:34-194)
- Counsel: practice for the revision pass (numbers in data; each system reads only what came before; one formula per behaviour; named seams). (decisions.md:923-930)
- The companion rule to "refactor religiously" (old presets load and seeds bake the same); unclear whose. (handoff.md:330-332)
- Builder's "Long" list: her reading of journal/vision/, not David's words. (handoff.md:231-235)
- Counsel: a one-page demo scope (one race, one biome, campsite to cottage, goblins at night, the blood sphere, no magic); a 30-minute demo loop (arrive, build, recruit a dwarf, trouble at night). (watch-list.md:39-49; where-we-are.md:102-120)
- Counsel: marketing (the pitch; the Steam page as one door, and a Steam page early; $200 shrinking the modding community; a written modding policy; game first, then mods, then the editor; phases from claiming the name to Early Access at $20 to 30; a streamer-friendly mode with seeds shown after the fact; a free world-gen toy; the honest story; the world map as a share button; "Claude drafts, David sends"; a demo built to produce a retellable story; a camera path tool; timeline guesses). (marketing.md:8-176, 192-247; where-we-are.md:102-120)
- Counsel: "millions of dollars" as the commercial aim. (where-we-are.md:77)
- Counsel: a trademark check, handles and a domain; commit and back up the art; AI disclosure; receipts (commit the art, export David's messages, record art sessions, back up off the machine, say it first on Steam); the art-in-git setup (deferred by David). (watch-list.md:115-135; how-its-made.md:58-83; handoff.md:297-329)
- Grok: name candidates (David took the Latin route himself). (conv:280-282)

---

Ref key. In journal/docs/: vision.md, decisions.md, jobs.md, objects.md, block-properties.md, data-files.md, release.md, simulation.md, quartile.md, placer.md, caves.md, rivers.md, water-inventory.md, regroup.md, far-view.md, dyno.md; review = review-2026-11-10.md; WG = worldgen.md; WR = water-rework.md. In journal/vision/: from-grok.md (David's own ideas to Grok), watch-list.md, terrain-controls.md, terrain-pipeline.md, proposal-peaks.md, projection.md, toy-factory.md, marketing.md, how-its-made.md, ores-by-depth.md, where-we-are.md, handoff.md; proposal = proposal-voxel-renderer.md; WS = water-from-the-start.md; conv = incus_artus_full_conversation.txt. msgs = Art/session 2026-10-24 - David's messages.txt. notes = notes/section notes.toml. mem/ = Claude's memory files for this project. Line numbers for simulation, quartile, placer, caves, rivers, water-inventory, regroup, far-view, dyno, proposal and review come from condensed readings and are approximate.

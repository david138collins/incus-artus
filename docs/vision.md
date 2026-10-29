# Incus Artus — Vision

What the game is meant to become, gathered from David's design brief, his answers, his notes vault
(`lorecraft/art/language/Incus Artis/`) and our conversations. `docs/decisions.md` records what's
been *decided and built*; this file records *where we're going*. Status tags as in decisions.md:
**[DECIDED]**, **[IDEA]** (David's current thinking, may change), **[OPEN]** (needs an answer).

---

## In one paragraph

An open-world **immersive survival and settlement simulation**, with horror that comes from how
enemies behave (see Threats). **Minecraft-style first-person building and exploring**, combined with
**Dwarf Fortress-style living settlements**. You can stay a lone adventurer or grow from a campsite
into a network of towns, moving freely between first-person play and an isometric management view.
The world lives a normal lifetime of procedural events and then drifts into an endgame as evil
grows, but it never pushes you too hard: you can ignore the world and live a Stardew Valley life
on a farmstead (until the horde arrives, maybe). Commercial, standalone, Windows first, data-driven
and moddable, with lightweight graphics so the computer's effort goes into making the world feel alive.

## What it improves on  [IDEA]

- Minecraft rebuilt from scratch with what's been learned since: the enhancements players broadly agree
  on from hardcore-realism mods, RLCraft and Vintage Story. Enough extra depth of simulation and quality
  of life that everyone can have their survival adventure their own way, with their own solutions,
  while keeping the joy of Minecraft.
- **Natural, logical solutions to everything.** No redstone: what a contraption does, a few farmhands
  working for you do. Moving 10,000 rock blocks for a castle is solved with labor, wagons and donkeys,
  not hours of mining and shulker boxes.
- **Tangible storage** is fun to look at, so needing a wagon and two donkeys for a huge load is fine.
- **Fair inventory balancing**, not strict realism: carrying several whole beds in the hotbar is the
  kind of thing nobody thinks twice about, and stays.
- **Races:** each is balanced to be fun to play; **humans are the standard choice**. Development starts
  with **dwarves** because their settlement behavior is the biggest thing being added to Minecraft.

---

## Story and setting  [IDEA]

- **The gods.** The Goddess of Fate is married to the God of Darkness; the Goddess of Life and Death
  is married to the God of Fire. While Darkness slept, Fire fathered children with both goddesses:
  Fate bore three boys and Life-and-Death three girls, one **dwarf**, one **human** and one **elf**
  each. The gods thought little of it and abandoned them on a planet.
- **Where the gods live.** Fire at the centre of the earth. Both goddesses inside a moon (the planet
  has **two moons**). Darkness inside a wandering planet on an elliptical orbit ("planet X").
  The gods aren't really local the way mortals are.
- **The awakening.** The children's descendants learned magic. Archwizards theorising a 4th
  fundamental force in their *thaumagenetic* experiments woke the God of Darkness, who discovered
  the infidelity and set out to murder all the offspring.
- **The player.** The God of Fire, angered, fathered a child with a mortal woman: the player
  character, 18, origins unknown, who wakes in a tent on a road. The world enters the **Age of Strife**.
  - **[IDEA 2026-10-26] The seventh child of Fire.** Probably never said out loud (it may be too much to
    state plainly). Instead, an **ancient mural of the six progenitors of the races** in a forgotten temple,
    perhaps the Temple of Darkness where the Necronomicon is found, lets the player work it out.
  - **[IDEA 2026-10-26] The God of Fire appears.** An irresponsible, irascible deadbeat of a father:
    irritated when you bother him, somewhat callous. Perhaps a stone face in his temple that speaks. In the
    end, a moment of him standing there like "that's my boy". (The obvious first shape: a quest for Fire
    and one for each goddess, then a final one to somehow defeat Darkness.)
  - **[IDEA 2026-10-26] Darkness tells you who you are,** when he tries to kill you in a boss fight, with
    some deep, dark words. A bit "Luke...".
  - **[IDEA 2026-10-26] Agents of Darkness sent to murder you.** Foreshadowing in the classic shape:
    why such interest in you? (David knew early on there'd be hostile settlements and a cult lair.)
  - **[IDEA 2026-10-26] The chosen one, without meddling.** Evoke the "chosen one" tropes without the game
    deliberately steering the player. They also explain what's unbelievable about the player: carrying
    five beds in your pockets, abilities and behaviour no NPC has, NPCs reacting to you differently. You're
    not an ordinary person, and the world treats you that way, so the game's conveniences read as part of
    who you are.
  - **Prophecies:** David hasn't planned any. They're the obvious move every fantasy author makes; use one
    only if it's ever the only way to make the story work.
- **Endings** (the game never becomes unwinnable). **[DECIDED 2026-10-26] These are end states of the story,
  not the end of the game:** finishing one finishes *that story* and leaves the world in a certain state
  (the dark path is the world's third end state). Whatever you choose, you're still just playing Minecraft.
  1. **The gods' quests:** complete a quest for each god and somehow defeat the God of Darkness.
     Evil settlements then stop trying: they defend themselves but no longer expand or pursue goals;
     ones that need resources (goblins need food) may wither, like a badly run village.
  2. **Main force:** become mighty among men, conquer the nations and rout evil by strength.
  3. **The dark path:** summon the God of Darkness, take control of the evil settlements and gain the
     power to enslave the world. Done with a **Necronomicon**, found either:
     - at the bottom of a nasty procedural dungeon under a unique **Temple of Darkness**, located from
       clues scattered around the world (Indiana Jones style), or
     - by finding the **nine scraps** of the only other copy (someone tried to burn it) in bookstores
       across all the kingdoms, then having intellect workers (wizards, sages, scribes in a library or
       laboratory) research and assemble them.
- **Tone:** Game of Thrones-coded politics and slow-building dread.

## Pacing: the Dungeonmaster  [IDEA]

- A **Dungeonmaster** system runs the world's story: it draws **event cards** from a deck. Events are
  procedural but form consistent beats across a playthrough.
- Pursuing the main questline makes the Dungeonmaster draw **more often**.
- You hear of distant events through **messengers and word of mouth**, e.g. "a whole town of 120 was
  slaughtered by an army of goblins; nobody thought they could organise like that."
- **Evil grows on its own.** Each hostile site type has its own mini-mechanics (a bandit hideout grows
  differently from a goblin den). Ignore them and they swell; after longer than most people play a
  Minecraft world, attacks on good and neutral settlements become urgent. You can still cleverly
  preserve a farmstead while cities burn.
- The world is big enough to go off into the wilderness and make a living.

## The arc of the Age of Strife  [IDEA, 2026-10-26]

- **It has already begun** a little before the player incarnates.
- **The world weakens itself.** The kingdoms of humans, dwarves and elves fight each other out of greed and
  anxiety as evil looms closer. Many are ruined, and ruined kingdoms are what evil settlements (goblin dens
  in caves and marshes, and the rest) raid more. A world exhausted by its own struggle is less able to resist.
- **Two stages:** first power concentrates among the strongest; then even they are overwhelmed, as the
  weaker civilizations are ravaged to poverty or extinction.
- **Left alone, evil wins.** Without the player's intervention the Age of Strife ends in evil's victory.
- **The unexplored world gets harder by itself,** with no level scaling. A den visited an hour in might hold
  3 goblins and a shaman. Found after 80 hours, one that has been raiding and multiplying might hold 20
  goblins, some high level, several shamans, a chieftain and an ogre living with them.
- **Variety stays wild throughout.** Any cave, at any time, could hold a really nasty kind of zombie.
- **Pace [OPEN]:** players invest very different amounts of time, so the speed of the decline is hard to
  judge. It could be driven by how aggressively the player pursues storyline items (pursuing the main
  questline already makes the Dungeonmaster draw more often).
- **[DECIDED 2026-10-26] No player setting for it.** The pace isn't chosen at world creation (no
  "length of the age", no world types to pick). **The player's choice is their actions:** what they
  pursue, fight, build or ignore is what sets the pace.

## The Goddess of Fate: the probability engine  [IDEA, architecturally important]

- "Fate" is the engine's code word for **probability modifiers**. Every random roll, check and chance
  in the game goes through one system that knows what *kind* of roll it is, and which way the player
  would obviously want it to go.
- Story progression can **twist the odds of almost everything**, so unlikely things become likely,
  seemingly naturally, all around the player. E.g. after ignoring the Goddess of Fate's call, strange
  things happen "as she swore they would".
- Its nature is **never disclosed** to the player: emergent, inscrutable, a little heavy-handed.
- Needs: every probabilistic event hooked to it and **categorised**.

---

## Modes and interface

- **[DECIDED] Personal / party view (first person).** Direct physical interaction. Inventory and
  crafting windows float over the world; you can click items within reach *in the world* (lay alchemy
  supplies out on a table).
- **[DECIDED] Isometric supervising / foreman view.** Region selection, labor orders, oversight,
  height-slice scrolling, see-through indicators on people.
- **[DECIDED] Full-screen manager.** Tabs: Inhabitants, Resources, Buildings, Active Jobs, Assignments,
  Settlements, World Map (see the mockup `lorecraft/art/bad art/three man.png`).
- **[IDEA]** Before a settlement is bigger than a farmstead, management happens through a **diary**
  that travels with you. Larger settlements need a **manager block**; larger still, an **advisor** you
  talk to in order to open the management screen.
- **[DECIDED]** One interface shell for HUD, overlays, dialogue, barter, inventory and manager.
  UI art at 640×480 (maybe 1280×960), integer-scaled, beveled borders, restrained colour.
- **[DECIDED] HUD bottom bar:** equip section · three glass spheres (left to right **blue energy**,
  **red blood**, **green stamina**) with a sloshing pixel-fluid simulation · quickbar.
- Pausing: probably only from the manager screen, or as an option.
- **[IDEA, David 2026-10-28] Talking by looking.** Talking to NPCs is mostly annoying because it takes
  too long, so most talk doesn't take over the screen:
  - **Look someone in the face to start.** They stop what they're doing and greet you with an offer
    ("Hi! Do you want something?"). The text floats over them in a bubble.
  - **Answer with gestures:** move the mouse to nod yes or shake no.
  - **Choices sit round their face.** Nod yes and they show options; keep looking at their face until
    the options light up, then move the mouse left, right, up or down to pick one.
  - **They react to staring.** Keep staring and it goes "Hi!", then "...yeees?", then "What do you
    want?", maybe "What is your problem?" or "What are you looking at?"
  - **Or the number row** [David]: the options are numbered 1. 2. 3. 4., and while you're looking
    someone in the face the number keys pick them (instead of whatever they normally do).
  - **Look away for too long and the talk just dissipates.** No "goodbye" needed.
    - **Walking off can be a dialogue event of its own** [David]: leave mid-conversation, look back, and
      the bubble over them says "... okay, see ya then". The world notices you left, without stopping you.
  - **Staring can start a fight** [David]: stare the wrong person in the face too long without saying
    anything and they take it badly, up to turning hostile. A fun aggro mechanic built from the same trigger.
  - **Only talk that needs more than gestures** becomes a cutscene dialogue.
  - Where the news comes in: `docs/simulation.md`, section 12 (news particles).
  - Counsel's notes [PROPOSED]: four directions means at most four choices at a time, which keeps
    talk quick (deeper choices nest). Still to settle: how long a look counts as "looking at someone"
    (so glancing round a market doesn't stop everyone), only one person talking to you at once, and
    how the mouse both turns your head and answers (perhaps the view settles on their face while you
    talk). The number row answers the keyboard question. If the number row is also the quickbar, the
    quickbar is borrowed only while a face is looked at, so pick a "looking at a face" rule that can't
    trigger mid-fight (e.g. not while a weapon is raised, or not with hostiles).

## People

- **[DECIDED]** Races: dwarf 1.5 m, human 1.7 m, elf 2.1 m (elves crouch through doorways).
- **[DECIDED]** NPCs: names from data, needs (work, eat, sleep, wander, socialise), happiness from work,
  rest and play. Recruitable; assigned quarters and jobs.
- **[IDEA] Inhabitants vs followers.**
  - *Inhabitants* belong to a settlement: you give orders and wait for them to be done.
  - *Followers* are your personal party: you can use their inventories and tell them how to follow
    (single file, fan out, stay close with weapons out, be aggressive, ...).
  - **[IDEA 2026-10-26] Followers are people in the world who can join your party.** Special inhabitants
    you control more directly, through dialogue.
  - **[OPEN] Who can follow you:** everyone, only people written into the story, or a blend. David's current
    leaning is the blend: any village might have a fighter or a rogue willing to leave home and follow you
    for a price, plus story followers you unlock through quests.
  - **[OPEN] How much control you have:** not worked out yet. At least a **posture** setting
    (e.g. sneaky, aggressive, attacking).
  - **[IDEA] They stay close.** Not like Skyrim, where followers run off to attack someone. They stay near you
    in a roughly circular area, like a scene from The Lord of the Rings.
  - **[IDEA] They watch your back.** You should *feel* the difference: with a party around you, you aren't
    as open to being hit from behind.
  - **[IDEA] A co-op guest is a follower** (see Multiplayer). When the friend is offline, their character
    carries on as a normal person in the host's world, a special one: the host can keep them in the party
    or tell them to hang out at their house.
- **[DECIDED]** RPG system for NPCs and player alike. Attributes (from the mockup): Str, Con, End, Int,
  Wil, Cha, Spd, Dex, Agi. Skills such as Athletics (modulates sprinting).
- **[IDEA] Food → spheres.** Each food has amounts of **protein, carbs and nutrition**. Variety matters:
  - **Carbs:** the biggest immediate stamina boost; also speed up blood regeneration and injury recovery.
  - **Protein:** does more for blood regeneration and injury recovery.
  - **Nutrition:** needed to get all your energy back each day. Low nutrition means weaker magic, lower
    charisma, and an energy sphere you can't fully refill.
  - Most food and farming comes over from Dwarf Fortress.

## Life gauges: blood, stamina, energy  [IDEA]

- There's no "health": it's **blood**, the red fluid in the middle sphere.
- **The spheres are fluid simulations.** Each sphere is a 64-pixel circle in the 640×480 UI layer, and
  each fluid pixel is a fraction of that gauge. It must be gorgeous eye candy (people should stand there
  shaking the mouse to watch it slosh) but stay light.
- **Blood:**
  - Most blows cause **immediate blood loss** (the pool splashes and reacts; pixels flash and vanish or
    do other things as they leave) plus **bleeding** over time that has to be dealt with.
  - **Blunt hits** cause no bleeding, but immediate blood loss and they also destroy some stamina and energy.
  - **Critical hits** can **crack the glass**: blood leaks out of the bottom and runs down in rivulets.
  - **Bandaging** stops the bleeding.
  - **Natural regeneration** shows as drops condensing at the top of the sphere, growing until they run
    down the glass into the pool.
  - **Health potions** are poured into the pool and react with it.
- **Injuries:** on top of the gauges, a wide range of injuries, each with its own impediments. You get
  grievously hurt quickly.
- **Stamina** drains as you do things. Some recovers; some is lost until you sleep.
- **Energy:** using magic makes blue pixels **sublimate** and vanish. Spells whose medium has gradients
  work at **full effect only with a full sphere**: a fireball cast at full energy does more damage than
  one cast with your last 10 blue pixels.
- **Effects in the fluid:** lightning attacks send arcs through the pools, vaporising pixels. Status
  effects in general show through changes in the fluids' behavior.

## Death  [IDEA]

- You wake in a **church or a ruined shrine** of a god, barely alive, with some of your things missing,
  not knowing how you got there or survived.
- **With living followers:** the local sim closes, and the world sim decides whether they could defeat or
  escape the encounter. If they got away, you wake somewhere else *with them*: when you fell, they grabbed
  you and got out.
- A **hardcore permadeath** option can be added later.

## Combat  [IDEA]

- **Lethal and committed.** Everyone deals and takes a lot of damage; getting hit is never trivial.
  You don't walk around with a weapon drawn; drawing it is a deliberate act.
- **Gestural attacks.** With a weapon drawn:
  1. Press left mouse to begin an attack. From then until release:
     - **your feet are committed**: you carry on with the movement you had (e.g. sliding forward);
     - **the camera mostly holds**: mouse movement turns it only ~¼ as much, and it drifts back to
       where you were looking when you pressed;
     - **a cursor moves** away from the screen centre, damped and smoothed, tracing the gesture.
  2. Release to send the attack.
  - **Which attack** comes from your movement direction and momentum at the press, plus the path of
    the mouse between press and release. Example, a lunging stab: move toward the target, press, pull
    the mouse down a few inches (the sword tucks in), then snap it back up to centre while releasing
    (the sword stabs forward, almost carrying the body with it).
  - The feeling: a bit like **billiards**. Decisive and tactical: footwork, positioning (different
    attacks need different movement) and timing (every swing spends time positioning, executing and
    recovering).
- **Blocking** works on timing: pressing at the right moment is one kind of block, releasing at the
  right moment another. To be worked out.
- Animation: possibly David's own cheap living-room motion capture for sword and axe arm motions,
  interpolated around the centre.
- The ADSR "stickiness" in movement (sharp starts and ends) is meant to make this readable.
- **How it gets built:** one stage of a swing at a time, each dialled in before the next is added,
  until the structure of an attack exists. It comes after there are creatures that can attack and
  block back, so well after the first dwarf and his behaviors.

## Threats and horror  [IDEA]

- The horror comes from **how enemies behave**, not from the art style. Example: sleeping at an inn on
  your travels, you wake to a scream from the other side of the building. A woman has been stabbed by
  a goblin. Another scream outside: they're everywhere, it's dark, they're two feet tall and weak but
  cunning, cruel and armed with knives. The idyllic medieval world shattered by real crime and violence.
- **No level scaling.** A weak, low-level player can meet very high-level enemies. There are weak
  zombies and really strong zombies.
- Threats must be taken seriously, so they stay engaging, pull you into the combat system, and reward
  taking part in what's happening in the world instead of only being a hermit.

## Magic  [IDEA]

- Spells use **blue energy**, which barely regenerates except while sleeping. Only dedicated magic
  users have the energy to rely on magic for more than a passing encounter.
- Energy also stands for how **sharp** you feel: low energy (two nights awake fighting) worsens
  charisma and intelligence checks. Other intellectual tasks may use it too.
- **Scrolls:** anyone can use them; very valuable and rare. Either equip a one-time (or time-limited)
  spell into an equip slot, or it acts immediately when read.
- **Tomes:** a container of ~6 spell pages; find them, have them made, or add a spell if there's space
  (very expensive). Equip and read the book to load a spell into an equip slot or the left hand.
- **Memorised spells:** a few learned spells cast from a tab in the personal screen (with status
  effects, armour, stats).
- Content lists exist: scrolls (Alliance, Blessing, Cartography, ... Wonder), potions, rings,
  spells, wands of every wood and metal (vault `lists/`).
- **REMIND DAVID when magic design starts** [IDEA 2026-10-26]: **"spell lego"** à la Noita, where you can
  concoct spells irresponsibly. Bizarre, useless, dumb and dangerous spells that are amusing to make and
  watch may be a great feature.

## Economy  [IDEA]

- **Largely for show.** The Dungeonmaster makes it look busy: caravans carry goods from where there's
  plenty to where there's little; villagers visit market stalls stocked from the warehouse or bakery,
  as if buying and selling, though no money really changes hands.
- Economic pressure can drive township decisions ("we have lots of resources; what should we build?").
- For the player it must be real: goods are for sale, and **earning money to buy a pickaxe is as
  viable as finding copper and smelting it as a hermit**.
- **[DECIDED]** Construction and production orders cost gold as well as materials.
- Tuned much later, in a living world.

---

## Activities

Like Minecraft, a mix of: exploring, gathering resources, searching for a specific thing, fighting,
building, tending chores, organising your things. Plus, as you grow: managing followers and
inhabitants, laying out regions and workshops, and the main questline.

## World  [DECIDED unless marked]

- Circle ~72,000 blocks across, ~70% ocean, continents inside. Quality over size.
- DF-style geography, smoother local terrain. Biomes are continuous fields, independent of chunks.
- Vertical: bedrock (Z 0) → thin lava lake with stone **podes** holding rare minerals → dark caves →
  surface → sky (Z 399).
- Geology: many stones, soils and ores. **Material × inclusion**; ores as overlays with percentage
  chances, density and grade fixed at "planet forming".
- Ores (vault): bismuth (bismuthinite), copper (native, malachite, tetrahedrite), gold (native),
  silver (native, galena, tetrahedrite), tin (cassiterite), zinc (sphalerite), iron (hematite,
  limonite, magnetite), lead (galena), nickel (garnierite), platinum (native).
  Alloys: bronze, hard bronze, black bronze, brass, sterling silver, iron, steel (with recipes).
- Trees (vault): alder, ash, beech, birch, elder, elm, fir, hazel, larch, laurel, linden, maple, oak,
  pine, poplar, spruce, walnut, willow, yew, apple, orange, pear. They **fall physically**, built from
  quarter logs.
- Hostile sites: goblin den, bandit camp, evil cemetery, witch house, mad scientist, troll cave, cult
  hideout, pirate lair; plus the unique Temple of Darkness and its dungeon.
- Creatures (vault `monsters`): livestock, wild animals, giant animals, creatures, human foes,
  humanoids, goblin types, plant enemies, undead, supernatural, extraplanar.
- Later: soil chemistry, irregular winters, emergent temperature, light, fog of war shared by the
  settlement.
- A full world-gen stack with deep control panels, tuned before shipping.
- **[IDEA] Big enough to be alone in.** Game modes with an empty world can come later.
- **[DECIDED 2026-10-26] Starting a game is just starting a game.** For now the player chooses nothing at
  world creation: no settings, no world types. The worlds are big and won't be overpopulated. Every
  world-gen parameter (about 210 already, thousands by release) is tuned by David and fixed in the release.
  They're all compiled when the game begins, behind the world-generation waiting screen of lore and eye candy.

## Two simulations: local and world  [IDEA, architecturally important]

See `docs/simulation.md` for the working design.

- **Local simulation:** the full simulation, wherever the player is personally present (alone or with
  followers) and in and around the player's settlements and their inhabitants.
- **World simulation:** a lite version of the same thing, running everywhere else from the moment the
  game starts. It approximates the same social outcomes on coarse ticks with summary math, no
  pathfinding: "a spreadsheet of what's going on out there", a bit like a game of Risk in the background.
  - **Villages have problems**, from malicious things nearby, from generated events (a criminal in town
    starts a detective and social questline), or from a dire economic condition. Each is a potential
    procedural quest; solving it earns reputation and a reward, and helps the village prosper and resist
    the forces of evil.
  - **Villages have local resources and trade for what they lack**, or appear to: trader caravans spawn
    from need and surplus.
  - **Villages raise guards and fighters** when they can and send them to raid hostile settlements.
    No battle is played out; the result is calculated and applied (new structures, changed inhabitants,
    "wealth" up after taking livestock). These abstractions drive lively, logical decisions that make the
    world feel real.
- The world simulation must be able to **hand the local simulation the world** wherever the player goes,
  and take it back when the player leaves.

## The world map  [IDEA]

- World generation needs a **world map first**: a very explicit, literal visualisation of the generated
  world for David, which becomes the map shown to the player when creating a new world.
- **The player's version** may be a pencil-drawn fantasy map: roughly what the world looks like, not
  exactly where you'll be, so you have to remember it.
- **The manager's World Map** is the real version, built only from what you've **seen, learned, heard**,
  and know through your settlements' people interacting with the world.
- **Dev world-gen suite:** play with noise, border detection, noise mapping, tectonic continent forming
  and so on, all as *data* on the map, before any terrain is generated.

### Three maps, three purposes  [IDEA, 2026-10-23]

- **The designer's suite** shows plain data, like it does now.
- **The create-world map** isn't the real map exactly. It's a picture generated from the real map's data.
  - **A circular viewport that rolls like a globe** as the player pans and zooms. It isn't really a globe, but
    it should feel globe-like as it scrolls through the circle, out to the whole circle of water.
  - **Ice at the poles** (to be added to generation).
  - Delicious pixel art with sophisticated little modern effects, interpolated through the 8-bit dark mode.
    From opening the game to starting one, the art is nostalgic for the beautifully crafted, atmospheric
    fantasy games of the 90s and early 2000s.
  - **Map stamps** (`Art/Game Textures/map stamps/`): 17×17 pictures (mountains, oak trees...). The centre
    pixel is the point where the stamp goes onto the map. In David's example they overlap back to front over a
    speckled ground, with rivers drawn in.
  - Making the map beautiful is one of the last things to do.
- **The in-game map** is literal, with useful overlays. It shows the regions you've explored or heard of, what
  they're called and where they are, even before they have terrain data. (Later.)

## The opening  [IDEA, 2026-10-26]

**The waiting screen is the intro.** While the world generates (about 10 minutes):

- **The world forms before your eyes,** from a toy version of the real Tectonic Sim built for spectacle.
  Set up right, the plates become panes of glass that explode, shatter and orbit back to the centre as a
  glowing ball of spinning triangles. For the player's new world this runs **in reverse**: the land appears
  as an explosion of pieces coming together, which fade as land and mountains push through, and the ocean
  fills in from the edge of the circle. Mountains rise and forests spread as erosion is compiled.
- **Animated slideshow art** tells the story in a few scenes, and shows what the player can do, and that
  they can take part or not.
- **A fantasy interpretation of the map** lets the player see the larger world **once**. Then they find
  themselves somewhere in it, **without knowing where**. They work it out by talking to people, or by
  exploring until they see a big feature or shape they noticed during the intro. (This is the create-world
  map of "Three maps" above; the player never picks a spawn.)

**Then, after all the buildup, it's just Minecraft.** You're standing by a road with a tent, a loaf of
bread and a blank diary. Birds sing and flit about.

- **Immediately accessible:** you can start the normal Minecraft routine straight away, a little more
  involved, like RLCraft or Vintage Story. Every task is a toy, fun for its own sake and not too hard.
- **But you know there's more.** By now the player knows that finding a village is where things go way
  beyond Minecraft. At first it should feel more like Skyrim or Daggerfall.

## What the story is for  [IDEA, 2026-10-26]

- **Its practical purpose** is context: a reason the world is full of people living, doing things and
  causing a commotion, and a clear way for the player to take control of their world.
- **It rationalises the behaviours that make the world alive,** the way Dwarf Fortress's do. But this is
  Minecraft, so it isn't random stories at all times: some events come at random from people's behaviours,
  and none of it is a barrier to just doing Minecraft things.
- **It must be completable.** Not so hard that you can't finish it. Completing it leaves you a world you
  can be rich in and build whatever you want in.

## Voyages (new game plus)  [IDEA, LATER]

- **Take a boat to a new world:** a new world is generated for you to explore, and you can bring people
  and cargo with you. The journey takes about 10 minutes, the same as world generation.
- **You can sail back.** The lite simulation runs everything forward by the time you were away, so
  settlements need to be set up to sustain themselves.

## Build order after the foundation  [DECIDED with Grok, 2026-10-22]

1. A tentative **Fate service** for all future probabilistic events.
2. The **two-scope simulation layer**: regions move into and out of the local simulation as they load and
   unload; the world simulation covers the rest.
3. A **save system** built for both simulations and Fate (basic save/load on the debug panel).
4. The **world-gen suite and visualiser**, which grows into the world-creation map and the manager map.
5. Tune world gen as data in the suite, without generating terrain.
6. Later: terrain, the first dwarf in the world and his behaviors, and much later the combat processor.

## Building, resources and labor  [DECIDED]

- Minecraft-scale blocks, 32×32 faces; the **quartile placement system** (octants, ¼-block bars, items
  on quarters and centres); structure-editor "temple" with wave-function-collapse village pieces.
- Few inventory slots, extended by backpacks, followers, carts and pack animals. Weight and volume;
  containers; staged mining (rock → boulder → block); logs processed on sawhorses and workbenches.
- Workshops are blueprints of stations; villagers do visible multi-step labor; warehouse capacity is
  what you build; regions are shapes; seasons matter (spring and fall wheat; 2-week growing cycles).
- **[IDEA 2026-10-26] Workshops of any size** [David], as in Songs of Syx: how many tables, shelves and
  workbenches you put in decides how many people can work there. Maybe a Vintage Story-like crafting
  system inside that. David's caution: don't drift too far from Minecraft (a jeweller's lapping disk and
  angle-guided stone holder are a long way from it). His question: if every workshop is a chair, storage
  and tables, what makes a jeweller's different from a carpenter's?
  Counsel's proposal **[PROPOSED]**, three layers:
  - **One signature station per trade** says what the workshop *is*: a jeweller's bench, an anvil, a
    loom. This is Minecraft's own answer (a villager's job comes from its job-site block: the smithing
    table, the fletching table, the grindstone), so it's familiar, not a drift.
  - **Generic furniture sets the size:** tables, shelves and seats decide how many can work at once and
    how much is stored, as in Songs of Syx.
  - **Tools laid on the tables add the detail:** the lapping disk, the dop stick, a loupe, as small items
    placed with the quartile system (items on quarters and centres). They make the workshop readable at a
    glance and let a better-equipped shop work faster or make finer things. Tools are items, so they can
    be bought, made, stolen or broken.
  The detail lives in objects you can see, not in more steps for the player to learn, which keeps it close
  to Minecraft.
  - **Two kinds of crafting, one set of stations** [David's observation, counsel's framing]: Minecraft's
    crafting is closer to Skyrim's than Dwarf Fortress's: *you* walk up to a station and make a thing.
    Dwarf Fortress's is *labour*: other people make things over time, in chains, and the question is
    throughput. Incus Artus has both, so the same jeweller's bench could be used by the player
    Skyrim-style (walk up, make a ring) and be a workplace for villagers DF-style (work orders, slots).
  - **[IDEA 2026-10-26] Crafting by list, not a grid** [David]: no Minecraft "tic-tac-toe box", just a list
    of what you can make from the resources you have. With nothing to memorise, more items become feasible.
  - **[OPEN] How much object variety the world has** [David: "a can of worms"]. Job items scattered about
    by workers would make places feel alive, but players will want to pick them up. Counsel's notes:
    items that matter to a system (a tool that speeds work, a sellable ring) are real items, owned by
    someone, so taking one is theft that people notice; pure clutter can be part of a station's look and
    not an item at all. And variety can come the way blocks already do (material × inclusion): a few drawn
    forms × materials × gems by palette swap, so one drawing makes many items. The art is one artist's
    time (`vision/watch-list.md`, item 4), so variety should go where it means something.
- Calendar: year = 4 seasons × 7 days; day = 15 min light + 15 min dark.

## Multiplayer and modding  [DECIDED]

- Not a goal. Later maybe LAN / host-a-friend scale; a Discord feedback pipe and dev drop-ins first.
- **[IDEA 2026-10-26] Hosting, as a selling point.** Never an MMO, never servers. A player **hosts** their own
  world and **invites guests**; it's only for playing together. Nobody ever walks into an empty "server",
  and continuity is never a problem, because the world belongs to the host. No server infrastructure for
  David to run. The taste of honest old software like StarCraft 1: you host a game that you and a friend own,
  between the two of you and no one else. The cozy D&D-table scale.
- **[IDEA 2026-10-26] Co-op, not multiplayer: the host is special, and it's their world you're invited to.**
  - **The host is always the party leader;** guests are always in the host's party.
  - **The host controls everything only one player can control:** time (sleep, pause, speed), the manager,
    the settlements.
  - **Guests have no presence in the story.** They fit in as just another person the host has met. They can
    have their own houses, but they're **inhabitants under the host**.
  - **Guests can't advance plotlines.** They didn't get the quests, and people won't talk story to them. A
    friend can *perform* a quest's tasks, but the host completes it.
  - **Why players will enjoy it:** Minecraft friend servers are full of friction over who's doing what,
    sleeping, and keeping strangers out. Here none of that comes up: it's the host's world, and you're a guest
    at their table. A primitive model in the best sense, like the earliest networked games.
- Moddable: text IDs, content packs as folders, readable TOML.

## Notes for later: the gods, volcanoes, darkness and crime  [IDEA, 2026-10-23]

David's notes, kept for general consideration later.

**The story.** The God of Fire lives at the centre of the planet. To him were born children of the Goddess of
Fate and of the Goddess of Life and Death. The latter's husband had long slumbered in Planet X. When his
elliptical orbit brought him round the star again, he was woken by a commotion of men striving for
understanding. Looking closely, he saw what resemblance was borne in these creatures of the world. Knowing
the infidelity, and with grim rancour at his disturbed rest, he swore an oath of revenge on this world. And
so it was that darkness came to the lands of men and mer.

**What follows for the world:**
- **Volcanism:** volcano biomes you must brave to complete the quest for the God of Fire. Possibly dinosaurs
  there. Definitely a structure in a lava lake that goes down and down.
- **Weirdness and darkness are environmental layers,** like rainfall and temperature.
  - **Weirdness** is "the gods have been here" clues. A volcano can spawn somewhere odd in a weird area, which
    will be fun.
  - **Darkness** means the darkness service is working there, above a certain threshold.

**What follows for people.** Both layers are baked into the life-simulation architecture NPCs use to make
decisions:
- **Urges** are meters that fill over time and become a desire for an action (a drink at the bar).
- **Pressure classes** push the thresholds down, so events and conditions drive changes in behaviour.
- **Criminality** is a natural meter. For most people it fills very slowly and is depleted by other factors
  besides satisfying it, so few ever get the desire to commit crime and get exposed to the Jobs that exist
  for doing crimes.
- **Deep in lands of darkness,** allies you thought were loyal might rob you and flee, from the evilness
  pressure and perhaps an Opportunity (a localised pressure that turns up through a chance encounter).

**The aim:** a system simple enough to run as a background spreadsheet that David can manipulate to create
vanilla procedural story structures, yet complex enough that the interactions become emergent and too
complicated to fully predict. Minecraft missions always led down a rabbit hole that could spin out of
control; now the code can handle an unknowable number of cases, in convolution, in believable and authored
ways.

---

## Open questions

1. ~~Horror~~: answered, see Threats and horror.
2. ~~Death~~: answered, see Death. Still open: what happens to your settlements in hardcore mode.
3. **Blocking and defence** details; ranged combat; the injury list and its impediments.
4. **The religion mechanics** for shrines and offerings (beyond Fate's hidden odds).
5. **Character growth:** how skills improve; where attributes come from.
6. **Dialogue and procedural quests:** how conversations work. Started: Talking by looking (Modes and
   interface) and news particles (`docs/simulation.md`, section 12). Procedural quests still open.
7. **Animals:** mounts (donkey, horses, wagons, mule carts), livestock, hunting. Started 2026-10-28:
   wild animals are a likelihood per area made with the terrain, real only in the Local sim or as pets
   (`docs/worldgen.md`, The stages after Mountains).
8. **Audio direction.**
9. **Release plan:** first-release scope, platform (Steam?), timeline.

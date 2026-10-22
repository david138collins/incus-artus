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
- **Endings** (the game never becomes unwinnable):
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

## People

- **[DECIDED]** Races: dwarf 1.5 m, human 1.7 m, elf 2.1 m (elves crouch through doorways).
- **[DECIDED]** NPCs: names from data, needs (work, eat, sleep, wander, socialise), happiness from work,
  rest and play. Recruitable; assigned quarters and jobs.
- **[IDEA] Inhabitants vs followers.**
  - *Inhabitants* belong to a settlement: you give orders and wait for them to be done.
  - *Followers* are your personal party: you can use their inventories and tell them how to follow
    (single file, fan out, stay close with weapons out, be aggressive, ...).
- **[DECIDED]** RPG system for NPCs and player alike. Attributes (from the mockup): Str, Con, End, Int,
  Wil, Cha, Spd, Dex, Agi. Skills such as Athletics (modulates sprinting).
- **[IDEA] Food → spheres.** Each food has amounts of **protein, carbs and nutrition**:
  protein feeds **red blood**, carbs feed **green stamina**, nutrition feeds **blue energy**.
  Most food and farming comes over from Dwarf Fortress.

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
- Calendar: year = 4 seasons × 7 days; day = 15 min light + 15 min dark.

## Multiplayer and modding  [DECIDED]

- Not a goal. Later maybe LAN / host-a-friend scale; a Discord feedback pipe and dev drop-ins first.
- Moddable: text IDs, content packs as folders, readable TOML.

---

## Open questions

1. ~~Horror~~: answered, see Threats and horror.
2. **Death:** permadeath? Carry on as a follower or heir? What happens to your settlements?
3. **Blocking and defence** details; ranged combat; how injuries are modelled.
4. **The religion mechanics** for shrines and offerings (beyond Fate's hidden odds).
5. **Character growth:** how skills improve; where attributes come from.
6. **Dialogue and procedural quests:** how conversations work.
7. **Animals:** mounts (donkey, horses, wagons, mule carts), livestock, hunting.
8. **Audio direction.**
9. **Release plan:** first-release scope, platform (Steam?), timeline.

# From the Grok conversation: not yet in docs/

Ideas David gave Grok (early 2026 to 2026-10-26) that no file in `docs/` records yet. Gathered on
2026-10-26 from `incus_artus_full_conversation.txt`. Most are from the Minecraft-mod days, so some
may be outgrown.

**None of this is decided until David says so.** For each item David can say: *keep* (it moves into
the right `docs/` file), *change*, or *drop*. Items already covered by `docs/` aren't repeated here.

---

## Settlements and rule

- **Settlement tiers grow with a management skill:** campsite → cottage → farmstead → (towns).
  Running one spends **management capital**, so a few hours in you might afford a campsite and a cottage.
- **Housing:** a campsite holds 4, a cottage 6, a farmstead 12. You can travel with 3 followers easily.
- **Perimeters** grow. A campsite near fish can have a fishing spot; one near deer, a skinning and tanning rack.
- **Outlying campsites:** villagers can be sent to build hunting or logging camps beyond the perimeter,
  and to build different kinds of roads.
- **Abandoned settlements keep going** in a crude way, like NPC villages, and wither or survive depending
  on what was built. A place with no allies, followers or owned villagers disappears from the manager.
- **Workshops level up** from blueprints (Songs of Syx style: a bakery needs tables, shelves, ovens).
  Blueprints exist because decorating is what's fun in Minecraft and clumsy in Dwarf Fortress.
- **Kingship:** a thriving player can be appointed king of a kingdom, control about 10 towns and build forts.
- **Elections:** with enough reputation in a town you don't control, gather signatures from 60% of the
  people, take them to the current leader and demand an election. People you've done quests for will
  probably vote for you. Quests appear only when chance has a villager spawn one.
- **Owning a house** in someone else's town makes that town's simulation count you as an inhabitant.
- **Couriers or pigeons** can carry general orders to another settlement you own, with a delay.
- **Being there matters:** only one full management simulation runs at a time; you must visit to direct a place.

## War

- **Military structure:** squads of 5 led by a captain; brigades of 20 (four squads) led by a lieutenant;
  armies of up to three brigades under a general. It takes time after the Age of Strife begins before NPC
  kingdoms have captains good enough to become generals.
- **Thin-sim war is like Risk:** men, materials and gold spent; raiders appear at your border with
  equipment from their village's means (a level-x blacksmith and native copper means copper ringmail
  and copper spears).
- **Player power:** a high-level player with magic gear can solo a 20-man squad or take a fortress, if
  they're good.
- **Squad orders:** outfit and train squads; patrol the town, guard an area, deal with criminals.

## Combat details beyond `docs/vision.md`

- **Three layers of combos,** each harder to land and to parry. Different weapons have different moves;
  skill makes better moves possible, or easier.
- **Gestures:** a straight line is one guard; other moves need a V in a direction. Riposte: parry
  straight to the upper right to block an attack from there, then trace a V that way and reverse. It
  hits unless the defender meets a smaller block window.
- **Impaling:** an impaled mob doesn't advance. Pulling out by stepping back staggers it (to deal with
  someone else); left-click and step forward hurts it badly. A stronger, heavier target can shove you off,
  staggering you both lightly.
- **Movement:** lots of tapping WASD, sliding a little. You control your body's momentum with a weapon
  drawn, so you have to think about position. NPC vs NPC fights use the animations but resolve simply.

## People and races

- **Dwarf women** look almost the same as men, a little narrower with thinner beards. They're a third
  of births and about a third less likely to want a job, so they're rare.
- **Dwarven surnames:** sons take the father's last name, daughters the mother's.
- **Name syllables** (dwarven male first names): thok thog thuth thak that thab than dor dar der erb erg
  erk ert naf nof not nok nod bur bul bud bok ber. Last names: yon tor thot tak gul gos kon kro kah lerb
  loff leth ledd dul dor derb tul.
- **Lifespans and growth:** elves longest-lived and most expensive (like the Protoss), dwarves a middle
  life and investment, humans shortest-lived, fastest to reproduce and most adaptable.
- **Where they live:** elf villages in forests, dwarf villages carved into mountains, human villages on plains.
- **Cultures as golden ages:** humans as Romans, elves as ancient Greeks, dwarves as Hittite / Anatolian /
  Urartian (squared-off, brutalist statues and reliefs).
- **Villagers carry one stack and hold one item;** some jobs need a bag (gathering cotton).

## World

- **A goodness/evilness layer** decides some sub-biomes; necromancers in towers spawn more in evil areas.
  (`docs/vision.md` has *darkness* and *weirdness* layers, which may have replaced this.)
- **Caves have monsters** that respawn slowly, so a cleared cave stays clear for a good while.
- **The bigger a goblin den grows,** the higher-level the goblins around it.
- **Villages level their ground:** a radius around a finished village smooths slopes between structures,
  fading over about six blocks.
- **River Run (Skyrim)** is the model for the first little village you come across.

## Storage and things

- **10 slots** to start; a backpack takes one and holds about 6. Books stack to 4 even if unique.
- **Pallets:** 1×1 holds 4 miniature stacks; 2×2 holds 16. Crates (1×1), bins (open 2×2), boxes, chests,
  coffers. Casks, barrels and jars hold one kind, so a barrel of cheese counts as one food item.
- **Items on the ground** can be stacked.
- **Item kinds** (draft): Food, Materials, Supplies, Tools, Weapons, Armor, Valuables, plus somewhere for books.
- **Mining:** a block makes 2 boulders and a rock; a boulder makes 3 rocks or one cut stone (¼ block).
  Ore drops piece by piece, more for skilled miners. Mining time takes a fraction of the block's
  remaining HP, so an inept dwarf can start a block and a master can finish it.
- **Trees fall** in a line: each log level slides farther than the one below, about one block of
  distance per block of height, and lands at a predictable spot. Falling logs break leaves. Leaves give
  sticks; big logs can be quartered by standing them up and chopping. The falling should be tunable live.
- **Dirt** is easy to mine, for you and for NPCs. Build player and NPC interactions side by side so their
  balance stays in view.

## The product

- **Three kinds of player:** short-term (instant gratification at hand at all times), medium-term (wants
  to beat the game), long-term (wants to live in it). Two clearly forking choices at any time: look for
  trouble or explore, or build something toward a slower, more cerebral power fantasy.
- **Builders get help:** stamp templates down as orders, have haulers fill a warehouse near your build site.
- **An editor edition** for modding, built from the in-game structure editor, sold alongside the game.
- **Speedrun a fun demo,** no expense spared. Depth is discovered through reviewers and YouTubers.
- **Alpha testers:** watch them, chat with them, read the notes they leave in game, later drop into their game.
  (`docs/decisions.md` has a first version of this.)
- **Pause/sleep:** if the player is in their quarters with the manager or the iso view open and nobody is
  panicked, they sleep automatically and time advances.

## Art direction

- **256 colours,** chosen by hand as they're needed (started with 144 after studying Diablo II up close).
  Full colour depth only for effects, so they look striking.
- **90s DOS fantasy:** palette-crushed, hand-dithered art for menus and product art. Single-pixel borders
  that read as 3D through colour change, 8-pixel button text like iron letters in stone with rust stains.
- **UI at 640×480,** scaled by whole numbers to the player's screen. Bitmap fonts.
- Tools David settled on: Grafx2 (palette group editing), Aseprite, LibreSprite, Photoshop CS6 for indexing.

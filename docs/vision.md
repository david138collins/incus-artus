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
- **[IDEA 2026-11-08] Names on file:** David's, for the world's lore. What each one names is still his to say.
  - **Lacrimae Mundi**, "the Tears of the World".
  - **Malleus Verbi**, "the Hammer of the Word". *Counsel:* it pairs with the game's own name. Incus is the anvil, and
    Malleus is the hammer that strikes it.
- **[IDEA 2026-11-08] Threads from Excalibur (1981) and Blake,** kept for later. David asked for them to be saved
  after Counsel's review of the film. They're echoes, not decisions:
  - **The player as an Arthur.** Uther's lust begets Arthur, who is taken away with his origins unknown. Fire's son
    by a mortal woman wakes 18 on a road. The God of Fire is Uther seen from the other side, the father who never
    shows up. Mordred, born of a forbidden union and coming for the king, sits close to Darkness telling you who you
    are.
  - **The land and the king are one.** A ruler's state shows in the land: a wounded king's realm turns to
    wasteland, and a healed one blossoms. This is the projection principle as lore. It could be literal, with a
    kingdom's health and the darkness layer showing in the land itself (orchards withering, rivers greying,
    weirdness rising).
  - **The Dragon and the word.** Merlin's Dragon is a hidden power everywhere at once, the metaphysical world, and
    the Charm of Making is the spoken word that wakes it. Those are echoes for *Malleus Verbi* (a hammer of the word)
    and *Lacrimae Mundi* (the world grieving around a wounded king).
  - **Magic passes as the age turns.** Merlin knows the old powers are giving way. Morgana takes the charm from
    him: knowledge of the word passes on whether the teacher wants it to or not. This fits the Age of Strife.
  - **Blake's forge.** Los, the blacksmith-prophet of imagination, forges the world's forms with hammer and anvil
    (*Malleus* and *Incus*). Blake also wrote that Arthur's acts are those of Albion, the giant who is Britain.
  - **Colour that means.** Boorman lit his forest an unnatural green and used gold and grey for meaning, not
    realism: chosen colours, as in the graphics goal.

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
  **Reaffirmed 2026-10-29** (David, from an old piece made during the Minecraft mod): "most of it
  stands". The sections and their names still seem right. The lists under them are just associations
  of words that might get used there, not decided contents:
  - *Inhabitants:* name, status, current task, occupation, roles, skills.
  - *Resources:* food, supplies, materials, furniture, tools, clothes, weapons, armor, valuables,
    containers, objects.
  - *Buildings:* hunting cabin, woodcutter, residence, warehouse, tavern, carpenter, jeweller,
    mechanic, pasture, ranch, market, farm, paper mill, potterer, house, inn, foundry, weaver, bakery,
    bowyer, stonemason, coaler, blacksmith, tailor, mine, forge, guard post, depot, shrine, temple,
    hospital, bathhouse.
  - *Active Jobs:* fell trees, cut stone, cooking, harvesting, planting, hunting, guarding, animal care,
    patrolling, hauling, fishing, woodworking.
  - *Assignments:* mayor, captain, manager, priest, constable, secretary, treasurer, warden, accountant,
    trader, scholar, historian, court, mage.
  - *Settlements:* campsite, cottage, farmstead, village, town, city, capital, outpost, fortress.
  - *World Map:* goblin den, bandit camp, evil cemetery, witch house, mad scientist, troll cave, cult
    hideout, pirate lair.
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
    **Two tiers** [David 2026-10-29]: most random and common dialogue is immersive, from looking at a
    face and gestures. Important conversations are **cutscene dialogue**: it captures the mouse and finds
    a nice place for the camera between the two of you. **So the switch itself tells you it's
    important.** Shopkeepers don't need it. Starting a quest does, so you don't misclick an option.
    **The camera rule** [David]: find the most open area next to them, so you and the other person
    fill about 2/3 of the screen, pretty close, so finding the spot isn't complex.
    *Counsel:* keep cutscenes rare enough that the signal stays true. The camera needs a
    rule for a good spot (side-on to both faces, clear of walls, with the line between you unblocked),
    and a fallback for cramped rooms.
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
- **[IDEA, David 2026-11-01] School, college and church:** "i think all juveniles can benefit from
  school, but there needs to be some kind of college and church system for research into magic and
  other things."
  - **School is for every child.** All juveniles benefit from it.
  - **A college and a church** carry research, into magic and other things.
  - *Counsel* [PROPOSED]:
    - **School:** a schoolhouse and a teacher (an assignment), giving every child a small start on the
      basics (reading, numbers, a trade's first steps), so a schooled village grows up a little better
      at everything.
    - **Research makes knowledge you can hold:** new spells, recipes, enchantments, maps, written into
      books in the settlement's library. Books are items (tomes, above), so knowledge can be copied,
      traded, stolen or burned.
  - **[David, the same day] An idiosyncratic split:** "i would say the college does alchemy and the
    church does more magic. they probably both have different kinds of magic and alchemy. i like an
    idiosyncratic split."
    - **The college leans to alchemy, the church to magic,** and each has its own kinds of both, so the
      line between them is particular, not tidy. The college's research is done by its wizards, sages
      and scribes (the intellect workers already on file, who also assemble the Necronomicon's scraps).
    - **No underlying conflict between them:** "idk about any underlying conflict between the two,
      specific conflicts possibly. or personal ones." Conflicts are specific or personal, between
      particular people or over particular things, not built into the two.
    - **Healing two ways:** "physicians and magic should both be methods of healing."
  - **Guilds** (David): "places for tradesmen of a type to gather and improve and socialize."
    *Counsel:* a guild hall is where a trade's people meet, train each other and drink together, so it
    serves skill and social needs at once (the needs above: work, socialise).
- **[IDEA] Food → spheres.** Each food has amounts of **protein, carbs and nutrition**. Variety matters:
  - **Carbs:** the biggest immediate stamina boost; also speed up blood regeneration and injury recovery.
  - **Protein:** does more for blood regeneration and injury recovery.
  - **Nutrition:** needed to get all your energy back each day. Low nutrition means weaker magic, lower
    charisma, and an energy sphere you can't fully refill.
  - Most food and farming comes over from Dwarf Fortress.
  - **[IDEA, David 2026-11-09] Fat and carbs are both fuel; the names are open.** "im unsure about 'carbs', because
    fat and carbs both give energy or fuel, so they should both refill that nutrition meter. i dont like nutrition
    meaning your (protein, nutrition, carbs) state and having its own name in its own set. 'fuel/carbs' from carbs
    could just spike faster and wear off quicker and 'fuel/carbs' from fat could just wear off slower. i guess i
    could just color them with no names if english has failed me?"
    - **Fuel comes two ways:** carbs spike fast and wear off quickly, fat wears off slowly. Both refill the same
      meter.
    - **"Nutrition" can't be both** one of the three and the name of the whole set.
    - Names, or colours only, [OPEN].
    - *Counsel* [PROPOSED]:
      - **Colours on the gauges, plain words on hover.** The spheres are fluid simulations, so fuel can show its
        difference without words: carbs a thin, bright fluid that drains quickly, fat a thick, golden one that
        drains slowly. A player sees which kind they're running on.
      - **Words, if wanted** (hover and item cards): *fat* and *starch* (an older word than carbs, and plain) for
        the two fuels, *protein* for mending, and for the third, from plants, herbs and variety, *vigour*. The set
        has no name of its own; it's just what's in the food.
  - **[IDEA, David 2026-11-09] Food by colour:** "i see red as being meat, or protein, or beans/nuts/eggs, essential
    for strength, constitution, endurance. i see blue as being vitamins/nutrition/micros, essential for INT, WIS,
    CHA. i see green as being starch/carbs/fat, for agility, speed, dexterity."
    - **Red:** meat, beans, nuts, eggs (protein). Feeds strength, constitution, endurance.
    - **Blue:** vitamins, the small goodness of plants and herbs. Feeds intelligence, wisdom, charisma.
    - **Green:** starch, carbs, fat (fuel, fast and slow). Feeds agility, speed, dexterity.
    - *Counsel:* this lines up with two things already decided:
      - **The three spheres** (HUD bottom bar: blue energy, red blood, green stamina). Each food colour fills the
        sphere of its colour: red protein mends blood, blue keeps energy refilling, green fuels stamina. So a
        player reads their diet off the gauges.
      - **The nine attributes** on file (Str, Con, End, Int, Wil, Cha, Spd, Dex, Agi). They fall into his three
        threes exactly. *Wil* stays [David: "willpower probably, since wisdom is ambiguous against intelligence and
        becomes judgmental if inspected"].
    - This answers the naming question above: the colours are the names. Red, blue and green on the item card, the
      words only on hover.
    - **Left for the player to figure out** [David: "i guess as long as food is labelled when you check the card,
      we can just not say anything and let the player figure it out"]. The item card names the food and what's in
      it. The colours, and what each one does, aren't explained anywhere.
  - **[IDEA 2026-10-30] Salt or herbs to cure food** (David): salt is an alternative to herbs for
    preserved food. Curing a piece of meat takes **2 herbs or 1 salt**. Herbs add nutrition; salt adds
    none. Salt can be used elsewhere without being critical. *Counsel:* so salt is the convenient
    choice and herbs the nourishing one, a real choice rather than a must-have. Salt comes from dry
    basins and salt deposits (`docs/worldgen.md`, "Strata"), which gives dry regions something to
    trade.
    **David, 2026-11-09:** "i think adding herbs and salt to the meat to cure it will raise the nutrition value
    relative its pure protein, although the overall strength of cured meat is weaker." So cured meat is better
    balanced than plain meat, but worth less in total.
- **[IDEA, David 2026-11-09] Food carries you through the year.** His notes, taken while the dyno ran:
  "i need to reconcile long days and solid years with food and crops for players to not be frustrated. days that
  are 30 minutes of sun and 30 minutes of night appeal to me. but 7 day weeks then seem excessive, even if we had
  weekly crops. 5 days is 2 1/2 hours play if you sleep." His way through: rather than shrinking the calendar,
  "make crops slightly time intensive but extremely rewarding to the player minecrafting alone out there, while
  also providing a wide diversity of ways to get foodstuffs and cook them into something edible, to encourage
  experiencing a variety of tasks ... to make time work for seasons and crops, theres got to be enough to get a
  player through a year by exploiting all the different games we give him to play."
  - **Days:** 30 minutes of sun and 30 of night (in place of the 15 + 15 on file under the calendar). Sleeping
    skips the night, so a day is 30 to 60 minutes of play.
  - **Crops:** slightly time-intensive but extremely rewarding, worth it to a lone player.
  - **Many other ways to food,** each its own small game:
    - gathering flowers, incenses and plants "for spells of fasting";
    - foraging wild plants and cooking them by many recipes with fat, plants and herbs into more nutritious,
      more efficient food;
    - hunting: "hunting animals should be a thrill, with spear throwing and arrows and traps for rabbits."
  - **The first animals:** deer, fish, rabbits, fowl, pheasant, boar. They give meat, hide and fat. No organs
    [David: "i dont think organs are necessary, theyre all just meat"].
  - **Cooking always pays:** "cooking more and more complex food should always pay off as the nutritional value
    and benefits always slightly more than just add together."
  - **Storage:** storing food properly prevents spoilage, which "makes you find and fire clay" (pots). It joins
    curing with salt or herbs, above.
  - **Hunger impairs, it doesn't damage:** "hunger is usually too aggressive in fo:nv survival mode and skyrim
    survival mode and others. you can go three days without food before you really start becoming faint, and it
    doesnt 'damage' you, it just impairs you. obviously starvation can kill you." That fits the spheres above:
    low food shows as an energy sphere that won't refill and weaker magic, not as blood lost.
    **But it has to hurt** [David, the same day]: "the hunger has to be bad with lots of impairments, like you
    will not want to be hungry ... no, you dont lose blood from it, although you may lose more when hit, bleed
    longer." So:
    - many impairments, enough that you don't want to be hungry;
    - no blood lost to hunger itself;
    - hungry, you lose more blood when hit and bleed for longer.
  - **Food is a loop** [David]: "food is a loop and fun loops will attract lots of people with shorter attention
    spans." It has to be a fun loop in its own right, not a chore between the other games.
  - **Seasons counted in days** [David]: "i like just counting days of seasons. we may only need two seasons."
    No weeks or months: a season is a number of days. Two seasons or four [OPEN].
  - *Counsel's notes* [PROPOSED]:
    - **The year's length in play.** At the calendar on file (4 seasons × 7 days), 60-minute days make a season
      7 hours of play (3½ if you sleep) and a year 28 (14). So one crop a season is a long wait, which is
      exactly why the other food has to carry the player between harvests.
    - **Winter is the test.** A season with no crops must be bridged by what was stored, cured and potted, plus
      hunting, fishing and foraging. That gives storage and clay their reason, and makes autumn a harvest rush.
    - **Weeks aren't needed.** Seasons can simply be counted in days (David agreed, above).
    - **"Slightly more than the sum":** each dish's worth is its ingredients' sum times a small bonus for each
      kind of ingredient it combines (fat, meat, plant, herb). That rewards variety and complexity without
      running away. A many-step meal stays worth cooking without one recipe beating everything.
    - **Fasting as a choice:** because hunger impairs rather than harms, fasting can be deliberate. A spell of
      fasting with burned incense could trade the body's strength for magic's. This ties food to the magic
      system.
      **David** [same day]: "practicing fasting to enhance spells or energy is a neat idea, but it should make you
      much more vulnerable to death in any fight as well as being crippling physically without very high levels."
      So fasting:
      - strengthens spells or energy;
      - makes you much more likely to die in any fight;
      - is physically crippling, unless your skill at it is very high.
      A real trade, for the dedicated. David: "fasting to cast harder spells is a glass cannon."
  - **[IDEA, David 2026-11-09] Cooking minigames, knives and butchery:** "i think adding almost cutesy cooking
    minigames would be good as long as the reward is enough and that cooking can get faster with chef knives for
    preparing ingredients faster, regular knives for skinning and quartering (i think the deer or boar would pop
    apart without any more gore than the blood particles from killing it, and the pieces would stack in the
    'every item is an inventory of like items' way, to be roasted (easy but hard to store) or butchered with a
    butchers knife (more quickly) into proper meat (slices/steaks) (you get more out of it) which can be cooked in
    more ways and stored more easily".
    - **Cooking minigames:** almost cutesy, as long as the reward is worth it.
    - **Knives by job:** a chef's knife prepares ingredients faster; a regular knife skins and quarters; a
      butcher's knife butchers more quickly.
    - **The carcass pops apart** into quarters (and hide and fat), with no gore beyond the blood particles from the
      kill.
    - **The pieces stack** as one slot of like items, as with blocks of every size ("One slot holds a material in
      every size", under Building).
    - **Two ways with a quarter:**
      - **roasted whole:** easy, but hard to store;
      - **butchered into proper meat** (slices, steaks): more meat from the same animal, more ways to cook it,
        and easier to store.
    - **The cut's name, and cozy food** [David, the same day]: "i guess the best way to divide an animal would be
      haunches or what? quarters are just 1 item unless we were doing too much i think. food should reward with a
      cozy feeling for some people who just think its fun to roleplay. so wording and appearance matters."
      - One kind of piece per animal, not front and hind quarters apart.
      - Food should feel cozy to players who cook for the fun of the role, so the wording and the look matter.
      - *Counsel* [PROPOSED], names from old cookery:
        - **Large game (deer, boar):** the carcass pops into **haunches**: a haunch of venison, a haunch of boar.
          It's the old word for a big joint to roast, and it sounds like a feast.
        - **Small game and fowl (rabbit, pheasant, fowl):** no cutting up; they come whole, as a dressed rabbit
          or a plucked pheasant, roasted on a spit.
        - **Fish:** whole, or filleted with the knife.
        - **Butchered:** venison steaks, boar chops, and thin strips for drying into jerky (which keeps).
        - **Fat by animal:** deer fat renders to **tallow**, boar fat to **lard**. Tallow also makes candles, so
          hunting feeds light.
        - **Cured:** a boar's haunch salted becomes salt pork or ham (salt or herbs, above).
        - **Dishes with old names:** pottage, stew, pie, pasty, sausage.
        - **The look:** warm colours, steam, and the sizzle and crackle of the fire, so a finished dish feels
          like a reward.
      - **The item card is plain** [David: "'itll see you through a cold night' is damp and clingy, just rabbit
        stew with (rabbit, potato, herbs, salt) somewhere on the items card"]. No flavour line: the dish's name and
        what went into it, as in "Rabbit stew (rabbit, potato, herbs, salt)". The coziness comes from the names and
        the look, not from sentimental text.
    - **[IDEA, David 2026-11-09] Over the fire, and how long food keeps:** "sticks can be consumed to put something
      over a fire, 3 for roasting a haunch, 1 for a bird fish or rabbit, and they visually appear in the campfire
      around it until its reused or removed. haunches roasted would be like 4 meals i guess but they would decay by
      2 days. smoking would last longer but require a smoker and more wood. curing with salt/herbs gets you a cured
      haunch i guess that is less nutritious but keeps for a while."
      - **Spits from sticks:** putting food over a fire uses sticks: 3 for a haunch, 1 for a bird, fish or rabbit.
        They show in the campfire around the food until reused or removed.
      - **A roast haunch** is about 4 meals and spoils in 2 days.
      - **Smoking** keeps longer, but needs a smoker and more wood.
      - **Curing** with salt or herbs gives a cured haunch: less nutritious, but it keeps a while. With the curing
        note above (herbs add nutrition, salt none), a herb-cured haunch would lose less than a salt-cured one.

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

## The start, tools and fire  [IDEA, David 2026-10-30]

- **Waking up:** you wake up in your **tent** with nothing but the tent, a **diary**, a **burlap sack**
  and a **magical loaf of bread**. The tent is a structure you can only crouch through and can sleep
  in, so you wake up crouched.
- **The magical loaf:** an ordinary loaf has 4 bites. This one appears once and never again in the
  game. It glows with a rainbow sparkle, and every bite heals and restores you entirely. It goes stale
  in a week and becomes normal bread.
- **Contextual text:** words from the UI that sometimes appear over things on screen when you see
  them, perhaps through a perception skill.
- **Stone tools:** useless to civilization, but craftable. Flint or stone makes **crude** tools that
  nobody would sell or want to buy, but you need them if you don't follow the path you wake up on.
- **The club is the first weapon you craft, and it should be wonderful.**
- **Fires, as in Songs of Syx** (David likes how its campfires, wells and markets work):
  - a **small fire**: 1 small log (a quarter log, from chopping up a whole log) plus some sticks;
  - a **larger fire**: 3 small logs and a bunch of sticks;
  - the **largest**: one large log, six small logs and a bunch of sticks.
  - **Cold NPCs want to warm up.**
- *Counsel's notes [PROPOSED]:* the quarter log is the quartile oak's own block, so felling, chopping
  and fires share one material chain. Warmth fits the impulses in `docs/simulation.md` section 13: cold
  is an impulse, a fire a place that answers it, so villagers gather round fires on their own (a fire
  as a zone, Dwarf Fortress style). The magic loaf is a gentle first lesson: the world gives you one
  miracle, then it's up to you.

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
- **[IDEA 2026-10-30] Pull and release** (David: "the better mechanic for attacks"). "Like playing
  billiards with a slingshot."
  - **Left click captures the mouse; pull back and release.** Which attack depends on **which quadrant
    you pull to and how far**.
  - **Aim and prediction:** if you were aiming correctly when you attack, predicting the enemy's
    movements, you hit.
  - **A slight quick movement in the attack's direction** helps players connect their blows.
  - **Hold too long and it's a feint:** past a second or two the attack fades, costs a small amount of
    stamina, and returns you to en garde. That takes part of a second. If they called your bluff, you're
    in a window of vulnerability; if they blocked nothing, they're vulnerable for a second. This also
    answers "what if the player just holds attack".
  - **The feel:** the player feels good. Tricky, but fun and easy once you get it.
  - *Counsel's notes [PROPOSED]:* show the pull with the gesture cursor (quadrant and how far), so the
    "slingshot" can be read before it's released; and let the feint's timing be a few sliders (how long
    before it fades, the stamina cost, the vulnerable window on each side), tuned on real fights. Enemies
    read the same pull, so a skilled enemy can feint you back.
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
- **[IDEA, David 2026-11-01] Spellmaking, enchanting and wands:** "i want you to be able to make spells
  and enchantments a bit like morrwind or daggerfall or noita, with unpredictable results. noita kept me
  playing like a crackhead just to make cooler wands. also wands work like this, ands staffs, you can
  put spells in them and the base item has modifiers too. fennling, like from ragnarok."
  - **Spells and enchantments are made**, as in Morrowind's and Daggerfall's spellmakers and Noita's
    wands, **with unpredictable results**.
  - **Wands and staffs hold spells,** and the base item has modifiers of its own, so the wand matters
    as well as what's in it.
  - **Fennling** (David): "fennling is the skill of transferring charge between wands. saves you from
    carrying multiple." A skill that moves charge from one wand into another, so you can drain spare
    wands into the one you carry instead of carrying several.
  - *Counsel* [PROPOSED]:
    - **The base item is a wand's frame:** wood or metal (the vault's wands list) sets its slots, its
      cast speed, how much it spreads or shuffles, and how much energy it holds, as Noita's wands do.
      The spells go in the slots.
    - **Unpredictable but not arbitrary:** each made spell or enchantment rolls once when it's made, so
      it's a real thing you can keep, trade or regret. Odd combinations misbehave in ways you learn.
    - **It's spell lego** (above, `vision/watch-list.md` item 18) and the FX Lab's pieces, so the
      spellmaker and the effects can share one system, drawn with the sprite particles
      (`docs/decisions.md`, objects, "Sprites for effects and spells").
- **[IDEA, David 2026-11-09] Magic's things, and where magic lives:** "potions, flasks (thrown potions), spells,
  rings, amulets, cloaks, scrolls, wands, staffs, tomes (reusable scrolls, slower than memorized from the tome),
  enchantments are kept in your pocket for things and cost a drain or reserve of energy, but gems can be enchanted
  and encrusted onto weapons and armor, inherently magical weapons and armor are either masterwork artifacts from
  an npc or items i add for quests or certain structure loots, and possibly only with astrological conditions for
  players practicing enchantment. blessings."
  - **The magic kit:** potions, flasks (thrown potions), spells, rings, amulets, cloaks, scrolls, wands, staffs,
    tomes and enchantments. They're kept in your pocket for such things, and cost energy as a **drain** (spent on
    use) or a **reserve** (held back from the blue sphere while it's worn or ready).
  - **Tomes are reusable scrolls:** cast from the tome, a spell is slower than one memorised from it.
  - **Weapons and armour take magic only through gems:** enchanted gems encrusted on them (above, Building:
    quality, material and encrusting).
  - **Inherently magical weapons and armour** are rare:
    - masterwork artifacts made by an NPC;
    - items David places for quests or as loot in certain structures;
    - possibly, for players practising enchantment, only under astrological conditions.
  - **Blessings** are on the list too (shape [OPEN]; there's also a Blessing scroll in the vault's lists).
  - *Counsel:* the reserve is a good fit for the blue sphere. A worn ring could hold part of it back, visibly
    (that part of the fluid locked, or a different shade), so wearing magic has a cost you can see, the way
    fasting does.

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

**[IDEA 2026-11-09] The bake belongs in this wait.** David: "i am willing to make the player endure a long
initial 1 time wait to prevent them from dealing with the chunk loading i have going on. we might have to run
around and juggle to get them to accept that with some contrived game starting activities, like a tiny Hazard
Course world that the game world can load in behind." The sight bake (about 6 to 7 minutes for a whole world at
build 92) is once per world, so it can fill in behind the intro above, and the player arrives with the whole
world already baked. *Counsel:*
- The world forming before your eyes is the activity he describes, and it's honest: the wait *is* the world
  being made.
- A small hazard-course world, to learn the controls while the real one loads behind, is the other option.
- If the player skips ahead, a misty dawn could hide whatever isn't baked yet, the mist lifting as the bake
  finishes. Fog is already built.
- David: "we can have a specific weather event for the game beginning that excuses us to get the world made.
  im not against a bit of stage magic." So the first morning's weather is chosen. *Counsel:* the fog's reach
  can follow how far the bake has filled, so the mist sits exactly at the edge of the world that's ready and
  lifts as it grows.

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
- **[David 2026-11-04] The fuller picture:** you construct or purchase and man the largest type of
  vessel, sail to the edge of the world, and enter an ocean-world map that you sail across "for a bit, not
  too long because it could get boring but you can roleplay if you want as captain". You arrive at a whole
  new generated continent or world, "with whatever and whoever you brought with you". You can return, and
  history runs what transpired in your absence.
- **Expansion packs** (David): "drastically different continents, like a more deserty and tropical one",
  to visit "once you're done with one world, or just have an itch to explore again".
- *Counsel, to assess:*
  - The crossing could cover the new world's generation (about 10 minutes above), so the wait is play.
  - A drastically different continent may be mostly world-generation settings and new blocks, data
    rather than new systems, if the World Compiler can reach that far.
- **How the voyage works** (David): "the ocean isn't real, the ship isn't moving". You're inside a very,
  very small world that is the vessel, and the ocean only looks like big waves and swells, and can move the
  screen "so that you're standing sideways when the ship 'rocks'". "almost a cutscene you can walk around in
  and do captainy things." When it's been long enough, you press a button and see your ship sail into the
  shore, drop anchor and lower a skiff. There the ship is just a motionless structure, and you row ashore
  on a regular small or medium boat.
- **Which land** (David): he can rebalance the world engine to make unique types of land, "and you could
  pick one and set sail". "an easy way to add content once a generation of players starts getting bored."
- **Oregon Trail mechanics, perhaps** (David): "fun 'oregon trail' mechanics to the voyage", though
  "recreating the mayflower with all the sick people and dying farm animals might be too far". How hard it
  gets is open.
- **Seasickness is welcome** (David): "making players seasick on a boat that isn't moving would make me
  happy, and i think they'd be happy about it too, looking back. thats half the reason Descent became such
  a classic. people were out there talking about how they fell out of their chair trying to play it."
- **Difficulty levels: don't assume one** (David): "adding a difficulty level would be much, much harder
  than it sounds at first, and only possible with basically a completed game. it is implemented so badly on
  nearly like, all games. it'd require extremely deep thought."
- *Counsel, to assess:*
  - The ship stays still, so people and routes on it work as in any structure, and the voyage asks nothing
    of marching moving things.
  - Rocking the screen: seasickness is part of the appeal (David, below). Whether it can be turned down
    at all, for players who couldn't otherwise play, is his call when it's tuned.
  - Arriving needs a good place on the new world's shore for the ship to stand.
  - Voyage events could come from the Goddess of Fate and the Dungeonmaster's pacing, so how a crossing
    goes depends on what you loaded and who you brought.

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
- **[IDEA, David 2026-11-01] One slot holds a material in every size.** "a storage system where you can
  have cut stone in a slot, and it stacks to 16; but that could be 16 whole blocks, 64 quarter blocks,
  or a mix of them; every block type which stores by itself, is like a hidden container of its own
  family in the inventory. that makes physical sense and improves quality of life, so when we're
  stressing the player on choosing what he carries, he doesn't feel like it's BS to need 3 slots to
  carry whole blocks, quarter blocks, and octblocks of 1 stone type."
  - A slot counts **volume**: 16 whole blocks' worth, so 64 bars or 128 octants, or any mix that adds
    up. It fits "weight and volume" above, and the pieces format already stores every size of a
    material alike (`docs/quartile.md`).
  - *Counsel* [PROPOSED; for when the inventory is built]:
    - **The family is one material in one form:** granite cut stone whole, as bars and as octants
      shares a slot; natural granite or granite brick is another family (the open "what kind of
      thing" question in `docs/decisions.md`, Blocks).
    - **Placing from a mixed slot:** placing a bar uses a bar. With only whole blocks in the slot,
      you'd cut one first (below), unless David wants the slot to cut for you.
    - **The slot shows its mix** at a glance (say "12¾"), with the sizes inside on hover.
- **[IDEA, David 2026-11-01] Cutting stone with a chisel, on screen:** "honestly i think the stone
  cutting action should be done on screen with a chisel on the ground, and so there's no reason
  octblocks shouldn't be a thing." So octblocks (lone octants of material) exist, made by cutting.
  - *Counsel* [PROPOSED]: lay a block on the ground and work it with the chisel in the world, where you
    can see it (the personal view's "click items within reach in the world"; the hammer and chisel of
    staged mining in `docs/decisions.md`). A clean cut loses nothing: a whole block gives 4 bars or 8
    octants. Only a piece cut by placing it against another is lost (`docs/quartile.md` section 12).
    Speed follows the tool, the material and the skill; for villagers, skill may also give a chance of
    spoiling a cut.
  - Until there's a chisel and an inventory, Shift + right click places a lone octant (Builder's
    stand-in).
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
- **[IDEA, David 2026-11-01] Quality, material and encrusting make an item:** "item quality, material,
  encrusting will make a big deal to the items strength, gems can be enchanted too for jewelry or
  encrusting weapons, armor, furniture, tools."
  - An item's strength comes from its **quality**, its **material** and its **encrusting**.
  - **Gems can be enchanted,** then set into jewellery or encrusted on weapons, armour, furniture and
    tools.
  - *Counsel* [PROPOSED]:
    - **Three layers, like a wand** (Magic, above): a form (sword, chair, ring), a material (bronze,
      steel, oak) and a quality from the maker's skill, then gems set into it. An enchanted gem works
      like a spell in a wand's slot.
    - **The item remembers who made it:** a maker's name and skill on fine work, as in Dwarf Fortress.
      A masterwork from a famous smith becomes a story, and a stolen one is noticed.
    - **The look comes cheap:** a few drawn forms × materials × gems by palette swap (the note above),
      so a gem-encrusted steel sword needs no new drawing.
    - **Gems tie mining to magic:** the gem list in the vault (diamond to onyx) becomes the enchanter's
      raw material, so the ladder of getting ore (`docs/decisions.md`, Blocks) feeds magic as well as
      metal.
- **[IDEA, David 2026-11-09] Quality first, within material; and sharpness:** "quality would be the biggest
  determinant of a weapon or armors worth, although poorly made adamantine or mithril weapons will still obviously
  be better than a masterful copper weapon. sharpness isnt durability, it wears down relatively fast and is restored
  relatively easily. skill can put a better edge on it that lasts longer and hits harder. a stone can sharpen it one
  step at a time from say 1/4 more slowly, where a grinding stone does it rapidly. dull weapons just lose
  effectiveness to a point. material plays a big role in how many steps of sharpness it has to maintain, and to use
  as you fight, so a steel sword may not need to be sharpened as constantly."
  - **Worth:** quality decides most of it within a material. Material still sets the range: a poorly made
    adamantine or mithril weapon beats a masterwork copper one.
  - **Sharpness is not durability.** It wears down fairly fast and comes back fairly easily.
  - **Skill makes a better edge,** one that lasts longer and hits harder.
  - **Sharpening:**
    - a whetstone restores it one step at a time, slowly;
    - a grinding stone does it fast.
  - **Dull weapons** lose effectiveness, down to a floor, never to useless.
  - **Material sets how many steps of sharpness** a blade holds before it needs work, so a steel sword can go
    longer between sharpenings.
- Calendar: year = 4 seasons × 7 days; day = 15 min light + 15 min dark. **[David 2026-11-09]:** days of 30 minutes
  of sun and 30 of night appeal to him more; how many days a season is now open (see "Food carries you through the
  year", above).

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

## Sound  [IDEA, David 2026-11-09]

- **David makes it himself:** Omnisphere (its modelled objects, its resynthesis of his own recordings,
  unlimited layering), his microphones and an RME Fireface.
  - Footsteps come from layered patches.
  - Fire comes from patches such as the recorded burning piano.
  - Repeated sounds are rendered as many variations, and the game picks among them.
- **No spoken words.** People and creatures make David's own grunts and hmms. Each kind of person or
  creature could have its own sound.
- **Music wish list** [David 2026-11-09, "a long ways off and dependent on how much buzz we can generate"]:
  smooth jazz from Macroblank, Oblique Occasions, or Emancipator & 9 Theory. Counsel's scouting:
  - All three are independent and reachable. Macroblank and Oblique Occasions are on Bandcamp and
    already collaborate with each other. Emancipator runs his own label, Loci Records.
  - Anything licensed must be free of uncleared samples.
- *Counsel:* ship sounds dry and let the game add the space. The sky light's measure of how much sky a
  place sees could also set how much a sound echoes and how muffled it gets in a cave.

## Figures  [IDEA, David 2026-11-09]

- **Minecraft-style figurines, slightly more detailed.** They have elbows (and presumably knees): "vintage
  story invented elbows and if i do everything they do people will complain so im leaving them elbows".
- **A Quake flavour** in the modelling: "i liked how quake looked, quick and crispy... quake flavor
  modelling might be a nice touch". *Counsel:* the Quake look is
  - chunky, angular silhouettes from few faces;
  - hand-painted textures drawn crisp, without smoothing;
  - animation in held poses snapping from one to the next, rather than smoothly blended. Quake 1 drew its
    keyframes without blending, which is much of the crispness.

  This fits the hard per-block light and the crunchy 2 × 2. Faceted pieces (helmets, weapons, a beast's
  skull) on block-built bodies would carry it without leaving the blocky family.

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

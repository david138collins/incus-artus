# Incus Artus — Simulation Design

How the game's living world is split into **simulation levels**, how the **Goddess of Fate** sits under
every random roll, and how all of it gets **saved**. Tags: **[DECIDED]** agreed with David ·
**[PROPOSED]** Claude's suggestion, waiting for a yes · **[BUILT]** exists in code · **[LATER]**.

The guiding idea (David): **only expose computational complexity when it's necessary**, for performance
*and* to keep a very large, complicated world organised in the software's logic. The world is folded up
like origami and **unfolds when discovered**.

---

## 1. Three levels  [DECIDED]

| | **Local** | **Intermediator** | **World** |
|---|---|---|---|
| What | The literal game world you're involved in | Places you're *explicitly involved with* but not standing in | Everything in the background |
| Examples | Where you are (alone or with followers); your settlement while you're there | Your settlement while you travel; a town where you took a quest or residence | Distant villages, dens, caravans you've never met |
| Exists as | Real blocks, real people and animals | Records with granular, economy-focused detail | Bookkeeping: a few numbers and dice rolls |
| How often | Every tick (30 a second) | Coarse ticks (default: every in-game hour) | Coarser ticks (default: every in-game day) |
| Movement | Pathfinding, physics | Distance ÷ speed | A dot on a loop that moves once a day |
| Fights | Played out | The generic fight resolution | "Village A needs a 16 to win this raid" |

- **[DECIDED] The names** [David, 2026-10-28]: the **Local sim**; the **Intermediator sim** (it exists
  to prepare the World sim's places for unfolding into Local); the **World sim** (or **Lite sim**).
  They were Local, Rich and Thin until 2026-10-28; the code, and older notes, may still say Rich and
  Thin. Where these docs say "the world simulation" for everything off-screen, they mean the
  Intermediator and World sims together.
- **[DECIDED] One place, one level at a time.** While a place is local, the world simulation keeps its
  hands off it and only sends requests.
- **[DECIDED] Local areas follow natural boundaries:** a settlement's area of direct control (which runs
  while you're controlling or just visiting it), or the area around you.
- **[DECIDED]** Intermediator and World use the *same rules* at different coarseness, so they approximate the same
  outcomes. Each rule is given "how much time has passed" and works it out in one go.

## 2. Hooks: why a place is in the Intermediator sim  [DECIDED]

- A place stays in the **Intermediator** sim while it has at least one **hook**: you took a quest there, took residence, own
  it, have followers there, and so on. Each hook says what it is, who holds it, and (optionally) when it
  **expires**.
- When the last hook expires or is removed, the place **drops back to the World sim automatically**. Nothing has
  to remember to clean up.
- Hooks are saved with the world, including their expiry, so transitions carry on correctly after a load.

## 3. Places, journeys and people  [BUILT as test versions]

- **Sites:** villages, towns, hostile sites, the player's settlements, ruins, the Temple of Darkness.
  Position, footprint (its area of control), level, hooks, **condition numbers** (population, wealth,
  food, fighters...) and **problems** (each a possible procedural quest).
- **Journeys:** trader caravans, couriers, travellers, wanderers, raiding parties. A journey follows a
  **route** of stops (chosen from what villages need and have, spread out by a distribution model).
  It keeps track of how far along it would be from travel speed and road distance, stays a while in each
  town, and goes on.
- **Meeting a journey [DECIDED]:** if you're on its route near where it *theoretically* is, it becomes
  more and more likely to be where you are, until it **enters the local simulation** (rolled through
  Fate). Until then it's just a dot.
- **[DECIDED] Everyone has a name from the very beginning.** People are stored compactly (a row each);
  their detail unfolds when it's needed. **[LATER]** with the name lists.

## 4. Expanding and collapsing  [BUILT as test versions]

- **Expand** (a place becomes Local): its records become real things. A population number becomes
  villagers standing there; a caravan's position becomes wagons on the road. Places never visited
  before get their terrain generated and their buildings placed from their records.
- **Collapse** (a place stops being Local): the local sim summarises back into the records.
- **No flicker at the edges:** a place becomes local at its footprint plus a margin, and stops being
  local only once you're a further distance away (both on the debug panel).
- Every level change is **announced** on the event bus ("site level"), so any future system can hook
  into expansion and collapse without the simulation knowing about it.
- **Collapsing in the middle of something:** when the player falls with followers alive, the local sim
  closes on a fight still going, and the world sim finishes it with a specific routine that's a
  **subset of the generic fight resolution** villages use when one raids another.

## 5. Fights on the books  [BUILT as a first version]

- One **generic fight resolution** for every fight that isn't played out: compare strengths, work out
  the d20 number the attacker needs, roll through Fate, then apply losses.
  - Example (David): "Village A needs to roll a 16 to win this raid. They rolled a 10, so they lost 4 of
    their 10 fighters."
- Results update the places: fighters lost, wealth or livestock taken, new problems.
- **News:** results can travel as news with couriers: "Lord Gherhart of Bluevalley crushes a bandit
  hideout, and the people rejoice", if they rolled a 20.

## 6. Death with followers  [DECIDED]

You fall down; the camera lies on the ground and slowly fades to black, maybe with a subtitle from a
party member ("[name]! No!"). If the followers are alive, they keep fighting for real long enough to reach
a death or loading screen, where you wait to see what happens while the fight drops to the specific
resolving routine (section 4). Then you wake in a shrine, or somewhere else with them if they got away.

## 7. Time  [BUILT]

- **The world clock** is part of the saved world (separate from the program's tick counter) and runs off
  simulation time, so pausing pauses everything and "TIME ×4" speeds everything.
- Calendar: day = 30 minutes (15 light + 15 dark), year = 4 seasons × 7 days.
- Intermediator and World updates are **spread out**: only a few places are updated per tick, never all at once.
- **Skip ahead** (debug): runs the world simulation forward days at a time, in one go.
- **[DECIDED] World generation simulates ~1,000 years of history** to some degree (simpler than Dwarf
  Fortress, but cleverer about unfolding), using the same rules and the same Fate engine.

## 8. The Goddess of Fate: one probability engine  [BUILT]

- **One engine, separate streams of dice:**
  - **Planet:** world generation's randomness, worked out from the world seed and the *position*, so the
    same seed always makes the same world, in any order. Never bent by Fate.
  - **Living world:** every roll while playing (and while history runs): hits, loot, events, encounters,
    raids, Dungeonmaster draws. **All bendable by Fate.**
- **Every roll says:**
  - its **category**, a dotted name like `world.raid`, `world.encounter`, `combat.hit`, `loot.rare`;
  - its **base chance**;
  - whether **the player wants it to happen**: yes, no, or doesn't care.
- **Twists** bend the odds for matching categories (`world.*`, `loot.rare`, or `*` for everything),
  each with a label (why it exists) and an optional end time. Two dials per twist:
  - **luck:** towards whatever the player wants (negative = against the player);
  - **strangeness:** towards things happening at all, whatever the player wants. This is how
    "unlikely things become likely".
- **Units:** +1 doubles the odds, −1 halves them. So a 1-in-100 chance with +3.3 becomes about 1-in-10,
  and nothing ever goes past certain. A chance of exactly 0 or 1 is never bent: impossible stays
  impossible.
- **d20 rolls** (for the bookkeeping fights): Fate decides success with the bent odds, then shows an
  honest-looking die face for the news ("needed 16, rolled 10").
- Fate's dice and twists are **saved with the world**, so reloading doesn't re-roll what was about to happen.
- Players are never told any of this exists.

## 9. Saving  [BUILT, first version]

Minecraft-style world folders, no separate save games:

```
worlds/<world name>/
  world.toml        format version, name, seed, when created and saved
  clock.toml        the world clock
  fate.toml         Fate's dice and twists
  sim.toml          sites, hooks, routes, journeys (the whole world simulation)
  player.toml       where the player is and looks, fly or walk, chosen block
  terrain.toml      the test terrain's settings (replaced by world-gen settings later)
  chunks/           changed chunks only, one compact binary file each
```

- **Each part of the game saves its own file.** A module adds its part with one line, like the settings
  groups: `app.add_world_state(...)`. A part that's missing from an older world starts fresh for that
  world's seed, and the panel says so.
- Chunks store block **names** (a little list per chunk), not numbers, so worlds survive block files
  being added, removed or renamed. A block that no longer exists loads as air, with a warning.
- Everything is readable text except chunks.
- **Save page** on the debug panel: world name, New world, Save, Load (a list of world folders), Open
  worlds folder. The game still starts in a fresh world.
- **[LATER]** Region files (many chunks per file), autosave, keeping the last few saves, saving local
  entities (people, animals, items) inside the chunks they're in.

## 10. Test stand-ins (temporary)

There's no world generation yet, so the simulation runs on three test sites and one caravan:

- **Test Village**, 150 blocks north of the centre (fly there with F).
- **Test Goblin Den**, 600 blocks east.
- **Test Market Town**, far away to the south-west, always in the background.
- **A caravan** that loops between the village and the market town, passing near the test world.

They follow **toy rules** (the den breeds goblins and raids, the village raises guards and raids back,
the caravan trades) only to prove the machinery works. When a place goes Local, magenta cubes appear as
stand-ins for its people; when it collapses, they vanish back into numbers.

## 11. The civilization lab  [IDEA, David 2026-10-26]

A lab for tuning the world simulation's behaviour, from kingdoms down to single people, the way the
terrain lab tunes the ground. By the time it's needed, terrain, climate and ecology are pretty much done.

- **Symbols on black.** Towns and people as high-contrast symbols, easy to read. No terrain, except
  overlays that matter to behaviour.
- **Fast.** Generate a world, then watch civilization be placed, expand and behave at very high speed:
  patterns a player would live through over 40 hours, seen in about a minute of watching the symbols
  go back and forth.
- **Rough pathfinding**, in the dark: journeys can find their way without anything being drawn.
- **Overlays that show intent:** where people are currently headed, as traced lines; the candidate sites
  a group weighs when settling a new place, so you can see what it's choosing from.
- **The point** [David]: being able to see what's right and what's wrong, really fast, over and over.
- **Emergent, probabilistic, ruled** [David]: outcomes come out of behaviour and chance, but enough rules
  govern them that undesirable outcomes become rare.
- **Truly unpredictable, held in bounds, like a tokamak** [David, 2026-10-26]: every NPC makes random
  decisions between its urges, weighted but random, and those drive results nobody can predict, quickly
  diverging. The goal is enough pressures that it stays mostly contained within reasonable bounds, most
  of the time.
- **Divergence** [David]: run the same world 300 times to see how differently its history can turn out,
  and 300 different worlds to see the spread across worlds.

Counsel's suggestions **[PROPOSED]**:
- **It runs the game's own rules**, the same Intermediator/World code and Fate (section 1). A lab with its own
  copy of the rules would tune a different game. Same principle as the terrain lab: same code for the
  lab and the world.
- **Two seeds: the world's and the history's.** Fate already keeps the planet's dice apart from the
  living world's (section 8). So a run is a *world seed* plus a *history seed*: keep the world seed and
  change the history seed 300 times for David's divergence runs; change both for 300 worlds. Any one run
  can still be replayed exactly from its pair, which is what makes a strange outcome worth finding:
  you can go back and watch it happen, and rerun it after changing a rule. This doesn't make the
  outcome predictable: nobody, including the game, knows how a history ends until it has been run, and
  the history seed in a real game is drawn fresh and never shown. Replay is only a *recording* of the
  dice, kept so a strange run can be studied. **[DECIDED 2026-10-26: the lab keeps replay.]**
  David's workflow: when something happens that should happen as little as possible, rerun hundreds of
  times to reproduce it, compare the runs where it happened with the ones where it didn't, and add
  another pressure, "another layer of magnets around the reactor". Keep snapshots as it runs,
  so you can scrub back along a timeline to the moment something went wrong.
- **Branch from a moment** [PROPOSED]: from a snapshot just before things went wrong, run the rest of
  history hundreds of times with fresh dice. It shows whether the bad outcome was already locked in by
  then (a missing pressure earlier on) or a fluke of the dice after it (a pressure missing at that point).
- **Compare side by side** [PROPOSED]: the runs where it happened against the ones where it didn't, the
  same graphs laid over each other, so the place where they part shows which pressure was missing.
- **Name the undesirable outcomes and count them.** Each is a check run on every history ("every
  kingdom gone by year 50", "a town founded in a swamp", "nobody left alive in the north"), and the lab
  reports how often each happened, with the runs it happened in, so one click replays it. "Rare"
  becomes a number to tune toward. Note what 300 runs can see: something that happens 1 time in 100
  shows up about 3 times; something that happens 1 time in 1,000 most likely doesn't show at all.
- **Ask anything why.** Click a town or a person to see its numbers, its problems, and the reasons
  behind its last choice (the candidate sites with their scores, the route it picked). `docs/jobs.md`
  already asks pressures to say why; this is the same idea for the whole world.
- **Graphs over time:** population, wealth, kingdoms, wars, how far evil has spread. This is the debug
  page the Age of Strife arc needs (`vision/watch-list.md`, item 13).
- **Many runs in one go**, with no drawing at all, using every processor core (runs don't depend on each
  other), and reported as spreads: e.g. "evil had won by year 300 in 12% of histories of this world, and
  in 4% to 30% of histories across worlds". One run shows a pattern; many show whether it's normal.
- **Speed:** 40 hours in a minute is ×2,400. The ~1,000 years of history at world creation (section 7)
  is about 14,000 hours of play, so the lab should run as fast as the simulation can, not at a set rate.
  World rules working out "how much time has passed" in one go are what make that possible.
- **Pathfinding needs a travel map:** the cost of crossing each patch of ground (slope, rivers, forest,
  roads), made once from the compiled world and saved. Routes are found on that, not on blocks.
- **Build it early in the Civilization stage**, starting with the test rules (section 10), so the first
  real rules are written straight into a lab that shows them.

## 12. News particles: how word gets around  [IDEA, David 2026-10-28]

You know nothing about a city on the other side of the world until news of it reaches you, carried by
someone.

- **The simulation already knows everything.** Whatever the world engine does (a raid, a wedding, a
  town founded) is an event in the chronicle (`docs/worldgen.md`, section 4), with an **event flag that
  expires** after a while. News only decides what the *player* gets to see.
- **A news particle carries no news.** It's a token with only an **area** on it: where it's from.
  - Couriers, traders and wanderers pick tokens up from the places they pass through, and carry them on.
  - Local people hold tokens of their own locale.
- **Using a token is chatting.** When someone with a token talks to you, the token grabs whatever in its
  area is marked as an event and hasn't expired, and fills in the dialogue's ad-lib from it: "Did you know
  the hamlet of East Ironfell Woods was attacked by an unusually strong goblin gang?"
- **What changes is visibility.** The event becomes visible to you: in the World Map, the chronicle, or a
  split between the two. Nothing is made up for the telling.
- **The player's chronicle as a diary** [David's leaning]: entries written as a diary, each with a link
  that puts a **temporary dot on the World Map** where it happened. That may be cleaner than trying to show
  past events on the map itself. (It fits the diary that travels with you before you have a settlement,
  `docs/vision.md`, Modes and interface.)
- David: cheap if the wiring is simple. A token is just a place id that a person carries.

Counsel's notes [PROPOSED, for David]:
- **Tokens copy, they don't move.** Telling you doesn't use it up, so word spreads out from a place like
  a ripple, and big news reaches you from several mouths.
- **News is as fresh as the telling.** Because a token holds a place and not an event, a trader from far
  away can tell you what happened there yesterday. If that ever feels wrong, the token could also note
  *when* it was picked up, and grab only events from before then. That's one number, and news from far
  away would arrive late, as it should.
- **When there's nothing to tell** (no live event in the area), the token could still put the place
  itself on your map ("I came through Ironfell, quiet place"), so travellers fill in the map even in
  peaceful times. A token whose events you've already heard could pick the next one, or pass.
- **How long an event stays news:** how remarkable it is could set how long its flag lasts. A hamlet
  raid is talked about for a week; a king's death for a year. Section 5's courier news ("the people
  rejoice, if they rolled a 20") is the same thing.
- **Later, if wanted:** rumours that drift as they're retold, or false news. Only if it's fun.

## Open questions

1. Default timings: Intermediator every in-game hour, World every in-game day? (Both are sliders.)
2. Should the first world-gen suite step also produce a list of village and site *types* as data files,
   so you can start writing their rules?

# Incus Artus — Simulation Design

How the game's living world is split into **simulation levels**, how the **Goddess of Fate** sits under
every random roll, and how all of it gets **saved**. Tags: **[DECIDED]** agreed with David ·
**[PROPOSED]** Claude's suggestion, waiting for a yes · **[BUILT]** exists in code · **[LATER]**.

The guiding idea (David): **only expose computational complexity when it's necessary**, for performance
*and* to keep a very large, complicated world organised in the software's logic. The world is folded up
like origami and **unfolds when discovered**.

---

## 1. Three levels  [DECIDED]

| | **Local** | **Rich** | **Thin** |
|---|---|---|---|
| What | The literal game world you're involved in | Places you're *explicitly involved with* but not standing in | Everything in the background |
| Examples | Where you are (alone or with followers); your settlement while you're there | Your settlement while you travel; a town where you took a quest or residence | Distant villages, dens, caravans you've never met |
| Exists as | Real blocks, real people and animals | Records with granular, economy-focused detail | Bookkeeping: a few numbers and dice rolls |
| How often | Every tick (30 a second) | Coarse ticks (default: every in-game hour) | Coarser ticks (default: every in-game day) |
| Movement | Pathfinding, physics | Distance ÷ speed | A dot on a loop that moves once a day |
| Fights | Played out | The generic fight resolution | "Village A needs a 16 to win this raid" |

- **[DECIDED] One place, one level at a time.** While a place is local, the world simulation keeps its
  hands off it and only sends requests.
- **[DECIDED] Local areas follow natural boundaries:** a settlement's area of direct control (which runs
  while you're controlling or just visiting it), or the area around you.
- **[DECIDED]** Rich and Thin use the *same rules* at different coarseness, so they approximate the same
  outcomes. Each rule is given "how much time has passed" and works it out in one go.

## 2. Hooks: why a place is Rich  [DECIDED]

- A place stays **Rich** while it has at least one **hook**: you took a quest there, took residence, own
  it, have followers there, and so on. Each hook says what it is, who holds it, and (optionally) when it
  **expires**.
- When the last hook expires or is removed, the place **drops back to Thin automatically**. Nothing has
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
- Rich and Thin updates are **spread out**: only a few places are updated per tick, never all at once.
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

## Open questions

1. Default timings: Rich every in-game hour, Thin every in-game day? (Both are sliders.)
2. Should the first world-gen suite step also produce a list of village and site *types* as data files,
   so you can start writing their rules?

# Incus Artus — Simulation Design (draft)

How the game's living world is split into a **local** (full) and a **world** (lite) simulation, how the
**Goddess of Fate** sits under every random roll, and how both get **saved**. This is a working draft
for David to review. Tags: **[PROPOSED]** Claude's suggestion, waiting for a yes · **[DECIDED]** agreed ·
**[QUESTION]** needs David's answer. When parts are agreed they move into `docs/decisions.md`.

---

## 1. The two scopes

| | Local simulation | World simulation |
|---|---|---|
| Where | Around the player (and party), and the player's settlements | Everywhere else |
| What exists | Real blocks, every person and animal as a creature in the world | Rows in a table: sites, groups, journeys |
| How often | Every tick (30 a second); heavy jobs every Nth tick | Coarse ticks, e.g. once per in-game hour (tunable) |
| Movement | Pathfinding, physics, collision | Distance ÷ speed along a map of routes |
| Fights | Played out, gestural combat | Calculated from strengths, then applied |
| Trade | Real goods in real warehouses and stalls | Need and surplus numbers; caravans appear from them |

**[PROPOSED] The rule that keeps them from fighting:** any one place is run by exactly one simulation
at a time. Inside the local area the world sim keeps its hands off and only *asks* (through requests,
like the event bus already does). Outside it, the world sim owns everything.

## 2. Places and things the world sim knows about  [PROPOSED]

- **Sites:** villages, towns, hostile sites (goblin den, bandit camp...), the player's settlements,
  ruins, the Temple of Darkness. Each has a position, a footprint, and **condition numbers**:
  population, wealth, food, safety, guards, morale, stockpiles by resource, plus a list of **problems**
  (each a possible quest).
- **Groups on the move:** caravans, raiding parties, armies, messengers, wandering monsters. Each has
  where from, where to, how far along, and what they're carrying or intending.
- **People who matter:** see the "people become real when noticed" idea below.
- **Each site type has its own small rulebook** (data files later): how a goblin den grows, what a
  farming village needs, when a village raises guards. The world tick runs those rules.

## 3. The handoff: expanding and collapsing  [PROPOSED]

- The world is divided into **regions** (proposed 512 × 512 blocks = 16 × 16 chunks, tunable). Like
  chunks, regions are a bookkeeping unit only and never shape the terrain.
- The **local area** is the set of regions near the player and near each player settlement
  (radius tunable on the debug panel).
- **Expand** (a region enters the local area): the world sim hands over what it knows. A village's
  population number becomes actual villagers standing in it, its stockpile number becomes items in its
  warehouse, a caravan on the road becomes wagons and donkeys on the road. Regions never visited before
  get their terrain generated and their sites built from their records.
- **Collapse** (a region leaves the local area): the local sim summarises back. Villagers become a
  population number again, items become stockpile numbers, the region's changed chunks are saved.
- **Where they meet is where the stories are.** The goblin inn scene works because the world sim
  decided goblins raid that village tonight, and the player happened to be sleeping there, so the raid
  happens for real in the local sim instead of being a calculated result.
- **Collapsing in the middle of something.** When the player falls with followers alive, the local sim
  closes on a fight that's still going, and the world sim must finish it with summary math: did the
  followers win or get away, and where did they carry the player? So the world sim needs to be able to
  take over an *unfinished* encounter, not only quiet villages.
- Moving a few regions in and out must never cause a stutter: expanding is spread over several frames.

## 4. People become real when noticed  [PROPOSED]

- Most villagers in the world sim are **just a number** ("population 120") until someone needs them.
- The moment a person is **seen, talked to, named in news or a quest, or has done something notable**,
  they become a **remembered person** with a name, skills and relationships, and stay that way forever,
  even after their village collapses back into numbers.
- This keeps the world sim cheap (hundreds of sites, not hundreds of thousands of people) while the
  people you've met stay the same people.

## 5. The player's own settlements  [QUESTION]

The player's settlement is where you spend the most effort, but keeping it fully simulated while you
travel far away costs a lot of computer and limits how many settlements you can have. Options:

- **(a)** Always fully simulated, like Minecraft's spawn chunks. Simple and exact, but expensive and
  caps the number of settlements.
- **(b) [RECOMMENDED]** Fully simulated while you're near. When you're far away it collapses into a
  **rich summary**: jobs and orders still progress by summary math ("the wall is 40% built", "3 days of
  bread made"), and it expands again, with the results built, when you come back or look at it through
  the manager.
- **(c)** Frozen while you're away. Simple, but makes the world feel dead and breaks "orders only".

## 6. Time  [PROPOSED]

- One clock. The world sim runs off the same game time as everything else, so pausing pauses both and
  "TIME ×4" speeds both.
- World ticks are spread out (a few sites per frame), never all at once.
- Calendar (already decided): year = 4 seasons × 7 days; day = 15 min light + 15 min dark.

## 7. The Goddess of Fate: one place for every random roll  [PROPOSED]

- **Two kinds of randomness, kept apart:**
  - **Planet forming:** world generation. Fixed by the world seed and position, the same every time,
    and *not* bendable by Fate (the mountains don't move).
  - **Living world:** everything that happens while playing: hits, loot, events, weather, births,
    the Dungeonmaster's card draws. **All of it goes through Fate.**
- **Every roll says:**
  - its **category**, a dotted name like `combat.hit`, `loot.rare`, `weather.storm`, `world.raid`,
    `dungeonmaster.draw` (categories will live in data so packs can add them);
  - its **base chance**;
  - **which way the player would want it to go:** good for the player, bad for the player, or neither.
- **Twists:** Fate holds a list of active twists. Each says which categories it touches (`combat.*`,
  `loot.rare`), how hard, whether it helps or hurts the player, why (for the debug page: "ignored the
  Goddess's call") and when it ends.
- **How a twist bends odds:** it shifts the chance on the "odds" scale gamblers use rather than adding
  percentages, so a 1% chance can become 10% and a 99% chance never goes over 100%. Each twist is one
  dial; several twists stack.
- **Debug page:** recent rolls by category (chance before and after Fate, result), active twists,
  buttons to add a test twist, and a "roll 1,000 times" tester to see the effect.
- **Save-scumming [PROPOSED]:** Fate's dice are saved with the world, so reloading a save doesn't
  re-roll what was about to happen.
- Players are never told any of this exists.

## 8. Saving  [PROPOSED]

Still Minecraft-style world folders, no separate save games (already decided). One world folder holds:

```
worlds/<world name>/
  world.toml        name, seed, game version, game clock
  fate.toml         Fate's dice and active twists
  world_sim/        sites, groups on the move, remembered people, the player's knowledge of the world
  player.toml       the player's position, inventory (later), settings for this world
  regions/          changed chunks only, one file per region, compact binary
```

- Readable text for everything except the chunks while we're developing, so you can open the files and
  look. Chunks are binary because there are a lot of them.
- **[PROPOSED] Debug panel:** a World page with New world, Save, Load (a list of world folders), and
  Open worlds folder. Starting the game still makes a fresh test world, as you asked.
- **Every saved file carries a version number**, so old worlds can be upgraded when the format changes
  instead of breaking.

## 9. What we can build and test before world gen exists  [PROPOSED]

There's no real terrain or villages yet, so each piece is tested with stand-ins:

1. **Fate** + its debug page. Testable on its own with the 1,000-roll tester.
2. **Regions and the local area**, with simple chunk streaming over the test terrain, so walking or
   flying far loads and unloads chunks. Debug overlay: region borders and which are local.
3. **The world sim table** with a handful of **test sites** (say "Test Village" and "Test Goblin Den")
   running toy rules: the den grows, the village raises guards, a raid gets calculated and logged. Its
   debug page is literally a spreadsheet of rows.
4. **The handoff:** walk to the test village and it expands (placeholder markers for its people and
   stockpile), walk away and it collapses back into numbers.
5. **A top-down debug map** of regions, sites and groups on the move. This is the first seed of the
   world-gen visualiser and the World Map.
6. **Saving and loading** all of the above.

## Open questions

1. Player settlements while away: (a), (b) or (c) above?
2. "People become real when noticed": does that match what you want?
3. Should reloading a world be able to re-roll Fate? (Proposed: no.)
4. Region size 512 blocks and the world sim's hourly tick: fine as starting values to tune later?
5. Later, with world gen: should the world sim run some **history** before you arrive (Dwarf Fortress
   style years of pre-history), so villages already have pasts, feuds and ruins?

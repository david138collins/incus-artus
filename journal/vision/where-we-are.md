# Where we are, where we're going

Counsel's picture of the whole project. Written 2026-10-26 from the Grok conversation, every file in
`docs/`, and the commit history. Updated after each counsel session.

---

## The game, in one breath

A first-person block world you can live in alone, Minecraft-style, which turns into Dwarf Fortress when
you take on people: from a campsite to a cottage, a farmstead, towns, a kingdom. A living world runs
around you the whole time at three levels of detail (Local, Intermediator, World), with every person named from
the start. Evil grows on its own. A hidden Goddess of Fate bends the odds, and a Dungeonmaster deals
event cards. Combat is lethal and gestural (feet committed, mouse traced, "like billiards"). The HUD is
three glass spheres of simulated fluid: blood, stamina, energy. The art is a hand-picked 256-colour
palette in a 90s DOS-fantasy style, Diablo II up close. Horror comes from how enemies behave, not how
they look. Three endings: the gods' quests, conquest, or summoning the God of Darkness yourself.

**Who it's for:** everyone for the first ten minutes (every block is a toy, trouble is always nearby),
the "beat the game" player for the questline, and the Dwarf Fortress veteran who wants to live in it
forever. Depth is found through reviewers and YouTubers, not tutorials.

**How it's made:** David designs, writes the rules, makes all the art and does the tuning. Claude writes
all the code: Rust and Bevy, data-driven, with a debug panel for every number that matters.

---

## Where we are (2026-10-26, day 6 of the Bevy project)

86 commits and about 27,000 lines of Rust in six days. That's a lot of progress, and most of it is
foundation that a lot of hobby engines never get right.

**Built and working**
- **The engine core** (10-21 to 10-22): fixed-tick simulation on a TimeControl clock with pause and speed,
  chunked storage, greedy meshing, texture arrays, walking with collision and momentum, block editing,
  the event bus, data files, Panini projection, the debug panel and presets.
- **The world's backbone** (10-22): the Fate service, the three simulation levels with hooks and
  journeys, the bookkeeping fight resolution, world saves. These are early versions running on test stand-ins.
- **People, first sketch** (10-22): urges that charge, pressures that lower thresholds, chance meetings,
  a test town of magenta cubes.
- **World generation** (10-23 onwards): the Editor Suite. Graphics Kit, Tectonic Sim (plates as rags of
  springs), bakes, the World Compiler, the World Interpreter, the terrain lab and its overlays,
  streaming, distant terrain out to the world's edge, the spawn picker, and now Additives (mountain grain
  and drift).

**Decided, not built:** jobs and occupations (`docs/jobs.md` has a build plan), the pipeline after the
landform (climate, rivers, erosion, soil and ore, then Ecology and Civilization), the quartile placement
system, the UI shell, combat, magic, the Dungeonmaster, 1,000 years of history.

**What you can do in the game today:** walk and fly over a tectonically generated continent and edit
blocks. Nobody lives there yet, and there's nothing to want.

---

## Where we're going

The build order agreed with Grok on 2026-10-22, and where it stands:

1. Fate service: **done** (first version)
2. Two-scope simulation layer: **done** (test version)
3. Save system: **done** (first version)
4. World-gen suite and visualiser: **done, still growing**
5. Tune world gen as data: **in progress** (Additives, then Lift)
6. Terrain, the first dwarf and his behaviours, much later the combat processor: **terrain done**, dwarf next

The near road, as `docs/worldgen.md` sets it: particle streaming off the mountains, then the mountain
synthesiser, then rework hills and ground detail around it, then grow the Additives, then Lift.
After that the pipeline still holds climate, rivers, erosion, soil and ore, Ecology and Civilization.

Counsel's view of that road is on the watch list (item 1). In short, world gen has a pull of its own,
and the demo needs people more than it needs a better planet.

---

## The road to selling it

### What "millions of dollars" takes

Steam keeps 30% (less once a game passes $10M). After refunds, regional pricing and sales tax, a
developer typically nets roughly half of the sticker price. At $25 that's about $12 a copy, so:

| Gross to you | Copies at $25 |
|---|---|
| $1M | ~80,000 |
| $5M | ~400,000 |
| $10M | ~800,000 |

For scale: Dwarf Fortress's Steam release sold about half a million copies in its first month (2023).
Valheim, a small team's survival game, sold well over ten million. Most Steam games sell a few
thousand copies. The genre can produce hits, and most attempts don't become one.

### Why this one has a real chance

- **Hooks that show up in a 15-second clip:** gestural sword fights, sloshing blood spheres that crack,
  a tree that really falls, goblins in the inn at night, a village working on its own. Short videos sell
  games now, and this game's best ideas are visual.
- **An audience that's starved:** "Minecraft and Dwarf Fortress at the same time" is a sentence a lot
  of people have typed into a search bar. Vintage Story, Songs of Syx, RLCraft and DF all proved parts of it.
- **A cost structure nobody else has:** one designer-artist plus AI coding. Burn rate is a subscription,
  so you can afford to take the time to get it right.

### What stands between here and there

1. **A demo with a loop.** Thirty minutes that are fun: arrive, build, recruit one dwarf, he works,
   something bad happens at night. That's the product test, and it doesn't exist yet.
2. **A Steam page early.** Wishlists are what Steam's algorithm reads at launch. The page can go up long
   before the game is done, as soon as there are a few screenshots that look like the game.
3. **Content and art throughput.** David is the only artist. Every block, model, animation and UI
   piece goes through one pair of hands (watch list, item 4).
4. **Early Access**, then years of updates. Games like this make most of their money over a long Early
   Access, growing with their community.

### An honest timeline

These are guesses, and they assume the pace holds:
- **A playable vertical slice** (the 30-minute loop, rough art): months, not weeks.
- **A public demo good enough for Steam Next Fest:** the better part of a year.
- **Early Access:** roughly 1 to 2 years out.
- **Millions:** only after Early Access, and only if the demo makes people talk. That's the one thing
  no amount of code guarantees, so it's the thing to test earliest.

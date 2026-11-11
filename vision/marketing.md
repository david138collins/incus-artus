# The marketing path

How Incus Artus goes from nobody-knows to a crowd waiting at launch. Written 2026-10-26.
David has no social media presence, and that's fine: the game gets accounts, not David's private life.

---

## The pitch

David's gift is the conversation that ends in "wow". It works because he starts from the **wish**, not
the game. The pitch should do the same:

> In Dwarf Fortress you build a living world you can never walk into.
> In Minecraft you can walk anywhere, but you're alone, and nothing is connected to anything.
> Incus Artus is both: build it, walk into it, live in it.

**Rule for every post, clip and page:** lead with the wish or the moment, never "check out my game".
"I made trees fall the way they really would" beats "devlog #4".

## The things people will share

Every one of these works without sound, in under 15 seconds:

- a tree falling in a line, logs sliding level by level;
- tectonic plates tearing like cloth and pushing up mountains (already exists);
- a dwarf going about his day without being told;
- the glass spheres: blood splashing, the glass cracking, drops condensing;
- a gesture swing landing;
- goblins coming through the inn door at night.
- **[IDEA, David 2026-11-01] The descent, in one unbroken shot:** "if we can get towns, bridges,
  roads readable from the sky like that, we're going to have some absolutely gorgeous material and we
  can put in a movie camera that flies above the world, and then flies down and you realize its
  minecraft, and it transitions without changing rendering or anything to the in game actual what
  you'll see while playing." And: "especially at night."
  *Counsel:* this is the trailer's opening and the Steam page's first clip. It starts on a continent
  from the air (David's shot of 2026-11-01, rivers to the sea, the mountain heart with its lakes),
  comes down onto a town lit by its windows and fires in a dark land, and ends in first person at the
  inn door. The point it proves is that there's no cut: the near and far are one world, which is the
  Metaphysics Engine shown in a single shot. At night it says the rest: light is precious, so every
  lit window, campfire and lair fire shows for miles. What it needs, all on file already:
  - towns, bridges and roads readable from far away (far builds, `vision/watch-list.md` 17);
  - lights visible from far away at night (`docs/worldgen.md`, cave mouths; `docs/decisions.md`,
    "Light is precious");
  - the near/far seam invisible while moving;
  - streaming that keeps up with a falling camera (David tests at 10,800 mph);
  - a camera path tool: a flight recorded or laid along a curve, played back smoothly, for
    screenshots and trailers. Small, and useful for tuning too.

  **Many worlds, then one** [IDEA, David 2026-11-01]: "we can also show different worlds appearing
  after the other from the air, or fade between them faster and faster as one becomes dominant as the
  camera falls to earth."
  *Counsel:* it says the other half of the pitch: every game is a new world, and each one holds up
  all the way down. The camera path tool makes it nearly free. Fly the same recorded path through
  several worlds, then cut or fade between the recordings in a video editor, so no engine work
  beyond the path itself. High up, the worlds change slowly, one continent after another. Lower down
  the fades come faster and shorter, until one world wins and the camera lands in it.

The goblins at the inn door matter most. Dwarf Fortress became legendary through **stories players retold**
(Boatmurdered). The demo should be built so that thirty minutes reliably produces a story worth retelling.

---

## Many doors, one house  [David's idea, 2026-10-26]

Two ways to sell it: **the toys** (immediate gratification; it just has to be good) and **the brochure**
(of all the forever worlds, this one stays freshest longest). And the scope has to be **deliberately hidden
from each audience**: someone shaking the mouse to watch the blood slosh can't also be told about armies,
kingdoms and a world generator with thousands of parameters. Each would be confused by the others.

So each audience gets its own door, built to look like a game they already recognise:

| Door | Audience | What they see | Where |
|---|---|---|---|
| **The toys** | everyone, Minecraft players | the spheres sloshing, a tree falling, a sword gesture landing | Shorts, TikTok, the first seconds of every trailer |
| **The adventure** | Skyrim and Daggerfall fans | an open world, mountains soaring into fog, the inn at night, the mural | the main trailer, screenshots |
| **The living world** | Dwarf Fortress, colony-sim and strategy players | villages going about their day, a kingdom, an army marching | longer videos, Reddit, YouTubers |
| **The machine** | procedural-generation and technical people | the Editor Suite, and the "cathedral of data" ad: the signal bus as light flashing between nodes, the camera pulling back like an EEG | devlogs, r/proceduralgeneration, X |

"What the hell is this game?" is what happens when people from two doors compare notes, and that's the
best kind of word of mouth, as long as **each door on its own is instantly clear**.

**Counsel's caution: the Steam page is one door.** Steam sorts games by genre and tags, and a store page
that tries to be all four confuses the algorithm and the visitor alike. The page should lead with one
recognisable promise (first-person survival and building, in a living world: the toys and the adventure),
then reveal depth further down ("...and it keeps going"). The other doors are the job of short videos,
devlogs and communities, which bring their people *to* that page.

---

## The editor and mods  [IDEA, 2026-10-26; counsel's notes]

David's idea: the game at about $20, **the Editor Suite sold separately at about $200**, and a **limited
licence to sell your mods** at up to half the game's price.

What's strong about it: the Editor Suite really is a professional-grade tool, and modded worlds are the
natural outlet for players who want choices the base game deliberately doesn't give (`docs/vision.md`:
starting a game has no options). Modding is how Minecraft and Skyrim stayed alive for over a decade.

What to weigh:
- **$200 shrinks the modding community.** Most modders are hobbyists and teenagers, and they're the ones
  who make a game last. A common pattern: modding through the data files is free for everyone (already how
  the game is built: readable TOML), and a paid tier adds the full suite. A price nearer the game's own
  might sell far more copies and grow the long tail. Worth deciding with real numbers later.
- **Paid mods: dropped (David, 2026-10-26).** No store and no licences. Mods run on the honour system;
  if someone ever sold one, David could act if he wanted to. (Background: in 2015 Valve and Bethesda opened
  paid Skyrim mods on Steam and withdrew them within days after a huge backlash.) Counsel's one addition:
  a short written **modding policy** when mods arrive (what's allowed, what isn't), like the usage
  guidelines Minecraft publishes, so "the honour system" has words behind it.
- **The order:** the game first, mods second, the editor as a product third. The editor needs players
  before it has buyers.
- **The palette grader** (above) can be the editor's first, free taste.

**The Graphics Kit as its own product.** David, looking at the kit on 2026-10-26: he'd have paid $75 for it
on the spot a week earlier, after spending $25 on apps that didn't work. That's one real buyer's signal,
and pixel-art tools are an established market (Aseprite itself sells on Steam and itch.io). A standalone
Graphics Kit (later with the Sequencer) could earn money and an audience *before* the game launches, under
the Incus Artus name. Cautions: it lives inside the game today, so it would need separating and polishing,
and it competes with the dwarf for time (`watch-list.md` item 1). Probably strongest once the Sequencer
exists, since animation is what nothing else on the market does this way.
David (2026-10-26): separating the tool sets and polishing them as standalones shouldn't be too hard.
Once dev streams start, people will ask for them; he'll either hand them out or sell them, because it
would be selfish not to let others play with them. The world-gen tools are a "good luck" case: it takes
days of tuning to get their best results, so they're less of a pick-up-and-play gift.

---

## The phases

### 0. Now: claim the name (an afternoon)

- The same handle everywhere: YouTube, TikTok, Reddit, Bluesky or X, a Discord server (empty for now),
  and a domain. Check the name first (watch list, item 11).
- A **clip habit**: install OBS (free) and record anything that looks good. F12 screenshots already exist.
  A folder of raw moments is the raw material for everything below.

### 1. Quiet building in public (now until the first dwarf works)

Post where the people who'd love this already gather, as a person sharing a project, not an advertiser:
- r/proceduralgeneration and r/VoxelGameDev: the Tectonic Sim and terrain lab belong here right now;
- r/gamedev's Screenshot Saturday and r/IndieDev;
- r/dwarffortress later, carefully (read their self-promotion rules first).

**Goal:** learn which moments make strangers say wow. About one post a week. No selling, no links to buy.

### 2. Testing the hooks (first dwarf until the vertical slice)

- Short vertical videos (TikTok, YouTube Shorts) of single moments. Their view counts tell you which
  hooks work, for free.
- Open the Discord, and a mailing list, once people start asking "where can I follow this?".
- A handful of trusted playtesters, in person if possible: David's conversations are his strength.

### 3. The Steam page (once it looks like the game)

When one biome, a dwarf and the UI have real art. Steam's "Coming Soon" page collects **wishlists**,
the number Steam's algorithm reads at launch. The capsule image is pixel art, which is David's strength.
A first trailer of about a minute, mostly the moments above. The page can go up long before the game is done.

### 4. The demo and Steam Next Fest

Next Fest runs a few times a year (usually February, June and October) and gives every demo a week
in front of millions of Steam users. Before it:
- send the demo to YouTubers and streamers who play Dwarf Fortress, colony sims and survival games;
  mid-sized creators reply far more often than famous ones, and they're the ones who make a game's first wave;
- make it easy to stream (a streamer-friendly mode). Players choose nothing when starting a world
  (`docs/vision.md`), so if worlds are ever shared, it would be by a seed shown after the fact, not typed in.

### 5. Early Access launch

Launch when the wishlists say there's a crowd, not on a date picked in advance. Price around the genre
(roughly $20 to $30). Every big update after that is a marketing event of its own.

### 6. The long tail

Mods and the editor edition, community stories, and big updates. Games like this grow for years after launch.

---

## The next game: *Incus Artus: The New God*  [IDEA, David 2026-11-01, "just thinking"]

David: "the next game could be called incus artus : the new god where we package a local llm as a
dungeonmaster to torment the player and hire professional artists to multiply the content, since what
we are making is a gaping maw that can turn very little content into, what we have so far."

*Counsel:*
- **The engine is the asset.** Little content goes in and a whole world comes out (the Metaphysics
  Engine, `docs/decisions.md`). Each piece of art a professional adds is multiplied across every world.
- **The title fits the story:** the player is Fire's child, a god's son among mortals.
- **Cheap now:** keep the Dungeonmaster's event cards as data with clear inputs and outputs (when,
  where, who, what follows), so a later Dungeonmaster, a local model among them, can choose cards
  and write their words without rebuilding the system.

## Ideas worth considering

- **A free toy:** release the world generator (plates, compiler, map) as a free little download or web
  toy. People who make worlds with it share them, and every shared world points at the game.
- **The honest story** sells: someone who can't code, making the game he's wished for since he was a
  kid, with every pixel drawn by hand in 256 colours. Present the AI-written code carefully (watch list, item 12).
- **The palette grader as a gift to pixel artists:** David found nothing in Photoshop, GIMP or their
  plugins that grades an image to a palette with real control. Pixel artists are a large, vocal community
  that overlaps heavily with this game's audience. A free (or cheap) standalone palette grader made from
  the Graphics Kit would earn goodwill and attention in exactly the right places, and it carries the game's name.
- **The world map as a share button:** the create-world map, the 90s-style pixel map, is something
  players would post. Make exporting it easy.

## A marketing command center  [IDEA, David 2026-11-08; for phase 2 or 3, name and placement his call]

David: "i don't know anything about marketing, social media, that kind of thing. i was thinking when that becomes
important i could make a marketing command center where claude API dragnets social media looking for places i
should market to, individuals with engagement online who would love the game, keep me notified if anyone starts
talking about incus artus online, etc." He has about $100 of Claude API credits a month.

*Counsel's notes:*
- **Listening (the easy part, worth it early):** a daily or weekly sweep for the name, read and summarised by
  Claude: Reddit, YouTube, Steam's discussions and reviews, Bluesky, Mastodon and the open web all have searches a
  small script can use. X, TikTok and Instagram are closed or costly to read; skip them or check by hand. A digest
  he reads with his coffee: who mentioned it, where, the mood, what they asked, a suggested reply he can rewrite.
- **Places (a research job, then a refresh):** subreddits, Discords, forums and hashtags where Dwarf Fortress,
  colony-sim, voxel and pixel-art people gather, each with its self-promotion rules. Better as a researched list
  every month or so than a constant dragnet.
- **People: only public creators** [David 2026-11-08: "im not looking for individuals with claude API. just forums
  and communities that would be interested in the first place, for me to go and post a thread with videos of the
  game or something. anyone who i emailed about incus artus specifically would be a video game reviewer, youtube
  lets play guy, etc. some public facing person making some money with it."] The dragnet looks for communities,
  not people. Emails go only to reviewers, Let's Players, streamers, curators and writers, people whose work is
  covering games and who publish business contacts for it: a list of those, ranked by fit and size (mid-sized
  reply most; phase 4).
- **The whole of it** [David 2026-11-08: "i just need a global map of the internet where i can identify areas that i
  should market to, and keep all the outreach and advertisement work organized, keep feelers out for any buzz, find
  places to lurk and listen for what players are asking for and what competition is offering, etc. it's not my
  forte."] Four parts, one place:
  1. **The map:** communities as territories, sized by members and activity, coloured by how close they sit to the
     game (colony sims, Dwarf Fortress, voxel, survival, pixel art, procedural generation), each with its rules
     on self-promotion and whether he has posted there yet.
  2. **The outreach log:** every post, email and reply. Where, when, what was sent, what came back, what's due
     next (a creator to follow up in two weeks, a community to post in once the next video is ready).
  3. **The buzz watch:** any mention of Incus Artus, with a digest and suggested replies.
  4. **Listening posts:** what players in those places keep asking for or complaining about, and what similar
     games are offering, announcing and getting praised or panned for. Read for design as much as marketing.
- **Its form** [David 2026-11-08: "i would basically use claude API as a little 'election campaign staff' type of
  assistant for keeping all the social media stuff i could beneficially engage with up front and obvious for me.
  not trying to make a game of thrones themed social media hub, i think a whiteboard window with notes, areas,
  updates, and logs would suffice"]: one plain whiteboard window (notes, areas, updates, logs). The staff work
  behind it; the board shows only what's worth his attention.
- **Claude drafts, David sends.** Nothing posts, replies or messages anyone on its own. His own voice is the draw.
- **Cost:** the reading and summarising is light use of the credits; the searches themselves are mostly free.
- **When:** listening can start the day the name is public (phase 0 or 1); the creator list before the demo
  (phase 4). A small page or a daily email is enough at first; a full command center can grow out of it.

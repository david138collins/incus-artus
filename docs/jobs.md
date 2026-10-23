# Incus Artus — People, Impulses and Jobs

How people decide what to do. Sorted from David's notes, 2026-10-22. Tags as elsewhere:
**[DECIDED]** David said so · **[PROPOSED]** Claude's suggestion, waiting for a yes · **[QUESTION]**.

The world depends on NPCs doing things of their own accord. **NPCs are creatures that crave jobs most of
all**, and if they get the ones they like, they're happy campers. The hustle and bustle comes from workers
solving their own problems through simple decisions and obvious default behaviors, before they give up.

**You can't fully control NPCs.** You can do your best to keep them productive, and plan your village around
productive routines and logistics (part of the fun of Dwarf Fortress), but it's never fully mechanical.

---

## 1. Impulses and thresholds  [DECIDED]

- Every person has **impulses**: work, eat, sleep, socialise, and more later. Each one **charges up** over time
  (motivation), like the momentum charge.
- An impulse becomes the person's **active goal** when its charge reaches its **threshold**. Example: 1,000
  units of motivation are needed.
- **Opportunities press the threshold down.** Something that presents itself lowers where the charge has to
  get to: with a chance to chat right there, 800 is enough. Discrete opportunities *modulate the threshold*
  instead of filling the charge.
- **Work** is the generic will to go out and do things, and the most common impulse. Following it means going
  to work, which means looking for a Job. The others come up less often and behave differently.
- **Randomness** acts on every impulse slightly, all the time.
- **Waves of weirdness** come by once in a while: they make people start doing things, or affect a number of
  people at random, just to stir things up.

**[PROPOSED] One dial for everything:** a "pressure" on a threshold (or on a charge rate) is the only tool
needed for:
- opportunities;
- likes and dislikes (a liked job's threshold is lower);
- schedules (night lowers sleep's threshold);
- moods and weirdness;
- Fate's twists.

Each pressure says what it presses on, how hard, why, and when it ends, so the debug page can show exactly why
Brom went to the tavern.

### Ideas from the first sketch  [DECIDED, to build]

- **Opportunities are very localised pressures.** Pressures in general can be extremely broad (a whole
  biome, a season, a mood); an opportunity is one right here, right now.
- **Show urges as vertical bars, like volume meters,** with a **channel strip** under each (its fill speed
  and normal threshold), like a mixing desk.
- **Different interactions satisfy different amounts.** Meeting someone he doesn't like may only take the
  socialise meter halfway down: other urges take over, but he wants to socialise again sooner. This works
  naturally for many habits.
- **Astrology** can be a factor to apply when there doesn't need to be any particular reason for something
  (why two people just don't get on).
- **Waves of weirdness become a field** defined by which kinds of chance events it acts on. Biomes with a
  high weirdness factor are more likely to produce a vanilla pack of chance encounters (Ma'aiq the Liar,
  the orc who desires an honourable death).

## 2. Opportunities and meetings  [DECIDED]

- **Opportunities present themselves immediately to NPCs through chance.** Example: two villagers walk to
  work and meet every day. When they meet there's a chance they stop and talk for a moment. They'll probably
  enjoy it at least a little, which satisfies their need to socialise. Because they see each other every day,
  they become **friends**.
- **In the bookkeeping:** check once in a blue moon how likely two people are to cross paths, times the chance
  of the chat. Rolled through Fate (`social.chat`).
- **The two levels drifting apart is good.** When you hang around a town, the full simulation won't come out
  exactly like the bookkeeping did, so things shift a bit as you settle in. That gives openings for
  dissatisfactions, events and job changes. The human eye can't see the lines between the tiles in all the
  little details of the hustle and bustle: like Minecraft, the world is seen by squinting, through imagination.
- **Exceptions:** some things must agree exactly between the levels, such as story advancement elements.
  Those are marked as such and handled exactly.
- **Self-balancing:** even a dwarf going nuts with several urges full will eventually sleep, as the pressure
  grows.

## 3. Jobs and who makes them  [DECIDED]

- **A Job is anything that needs to get done.** Many kinds of things can create jobs that apply to many kinds
  of entities, wild animals included:
  - a **village** makes lots of jobs;
  - a **kingdom** makes courier jobs, so someone runs from town to town and brings you news of Remarkable
    Events;
  - a **goblin den's** jobs are a list of heinous crimes that fill themselves in opportunistically.
- Where jobs come from:
  - **orders** (dig here, build this, make 5 chairs);
  - **settings** (a workshop set to keep wood stocked posts "haul wood here");
  - **opportunity**.
- **Idle people take jobs on their own.** Ten people doing nothing and a dig order: they find pickaxes if they
  can and start digging.
- **[PROPOSED]** Each job says **who made it** (the village, the kingdom, the den) and **who can take it** (its
  members, anyone, couriers only...).

## 4. Occupations and assignments  [DECIDED]

- You can **assign a worker to a workstation** (Brom, carpenter, in workshop 3). Going to work, an assigned
  worker **checks their station first**.
- **Each Occupation has a short list of places to look** for what it needs when it isn't where it normally
  is. So the computer isn't really trying hard to solve it: it's a few obvious scavenging behaviors, not a
  flowchart. Example for a carpenter needing wood:
  1. my workshop's storage;
  2. the warehouse linked to my workshop;
  3. other warehouses whose filters take wood;
  4. **the settlement's job list:** is any job carrying wood? A "haul wood to warehouse" job on the ground near
     the chopped trees: **my job overrides it**. I take the wood straight to my workshop and make the chair.
  5. give up.
- Other tasks may not have such persistent problem-solving set up. But the player needs to get things done,
  with some reliability from happy NPCs.
- **[PROPOSED]** The lists are data (one per occupation), written by David, like the block files.

## 5. Giving up isn't the end of the world  [DECIDED]

- A carpenter who can't do his job **gives up and looks for other work**: mining or construction jobs on the
  board, or another workshop with work orders and nobody assigned to that slot.
- The settlement's **player-visible event log** gets a line: *"Durthon Heffeg gives up trying to [work order]."*
  It doesn't say what he did instead. To find out, look him up in the Inhabitants tab and jump to him on the
  map, or check his "current activity" column.
- **[PROPOSED]** He doesn't retry the same order for a while, unless what he needed turns up (a "wood stored"
  announcement wakes everyone waiting for wood).
- **[PROPOSED]** Whoever claims a job, an item or a station **reserves** it, so two people never walk to the
  same log.

## 6. Likes, dislikes and happiness  [DECIDED]

- People **take jobs they don't prefer** if that's the best option they have. They get less satisfaction, and
  "have a drink at the tavern" overrides it more often (proposed: a disliked job presses down the thresholds
  of leisure impulses).
- Doing jobs they like makes them happy campers.

## 7. Hauling  [DECIDED: its own discussion later]

Warehouses and hauling get their own in-depth design, to be efficient for both the player and the NPCs. Until
then, hauling jobs are simple: take this from here to there.

## 8. The three simulation levels  [PROPOSED]

- **One set of rules, given however much time has passed.** Each person has a current activity with time
  left. The same code runs **every tick** when Local (a few seconds at a time) and **every in-game hour** when
  Rich (an hour at a time).
- **Local:** later, bodies walk and carry for real; the decisions stay the same.
- **Rich:** your settlement while you're away. People, the job board, and progress keep going as records.
- **Thin:** no job board. Rare bookkeeping: meetings and friendships, moods, and production as numbers.
- **[DECIDED] Everyone has a name from the very beginning.** Person records are small, so a whole world of them
  is cheap.

## 9. What gets saved  [PROPOSED]

- Every person: name, faction, home, occupation and assignment, impulse charges, pressures, likes,
  relationships, current activity and job, "gave up on" list.
- Every job: who made it, who can take it, source, what it needs, urgency, claimed by, progress, why it's stuck.
- Places: workshops (slots, work orders, linked warehouse), warehouses (filters, contents), piles on the ground.
- The settlement's event log.
- **Not saved:** reservations. They're worked out again from the claimed jobs when the world loads.

## Build plan  [PROPOSED]

Before world generation, so everything after it is built on this. Each stage ends with something you can watch
on the debug panel, with magenta stand-ins at the places people are.

1. **People and impulses:** person records, charges and thresholds, pressures, randomness, waves of weirdness.
   A **test settlement** on the test world with about 8 people. A **People** tab with each person's impulse
   bars and what's pressing on them.
2. **Jobs:** job boards, who made them and who can take them, orders and settings, occupations and workstation
   slots, work orders and test recipes, the occupation search lists, the haul override, reservations, giving up,
   the event log.
3. **Meetings:** chance meetings, chats and friendships, in both the full and bookkeeping versions.
4. **Levels and saving:** the same rules Local and Rich, the Thin bookkeeping, and all of it in the world save.

## Content David will need to provide (placeholders until then)

- **Name lists** for each race (given names, family names). Until then people are "Villager 1", "Villager 2"...
- **Occupations**, and each one's short list of places to look.
- **A few recipes** (what's made, from what, at which station, how long).
- **Impulses:** the list, and roughly how fast each charges.

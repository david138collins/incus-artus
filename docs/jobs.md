# Incus Artus — Jobs (draft)

How people decide what to do. Sorted from David's notes, 2026-10-22. Tags as elsewhere:
**[DECIDED]** David said so · **[PROPOSED]** Claude's suggestion, waiting for a yes · **[QUESTION]**.

The world depends on NPCs doing things of their own accord. **NPCs are creatures that crave jobs most of
all**, and if they get the ones they like, they're happy campers. The hustle and bustle comes from workers
solving their own problems through simple decisions and obvious default behaviors, before they give up.

---

## 1. Impulses  [DECIDED]

- Every person has **impulses**: work, eat, sleep, socialise, and so on. Each one rises over time, and the
  strongest one decides what they set off to do.
- **Work** is the generic will to go out and do things. It's the most common impulse and fills most of the
  day. Following it means **going to work**, which means **looking for a Job**.
- The other impulses come up less often and behave differently (eat finds food, sleep finds a bed).

## 2. Jobs  [DECIDED]

- **A Job is anything that needs to get done for the faction someone belongs to.** Each faction has a
  **job board**.
- Where jobs come from:
  - **Orders:** dig here, chop those trees, build this blueprint, make 5 chairs.
  - **Settings:** derived from workshop and warehouse settings. A carpenter's workshop set to "keep wood
    stocked" posts *haul wood here* jobs when it runs low; loose items post *haul this to a stockpile*.
  - **Opportunity:** for hostile factions, the jobs are a list of heinous crimes that fill themselves in when
    a chance appears (steal the livestock, burn the barn, kill the guard). **Goblins see jobs exactly the
    way villagers do.** That keeps hostile expansion in the same cheap bookkeeping as everything else:
    named people, job assignment, derived equipment, and borders that become real when you arrive.
- **Idle people take jobs on their own.** Ten people doing nothing and a new dig order: they find pickaxes
  if they can and start digging.

## 3. Assignments  [DECIDED]

- You can **assign a worker to a workstation** (Brom is the carpenter in workshop 3). Going to work, an
  assigned worker **always checks their station first**.
- Anyone without an assignment, or whose station has nothing to do, takes the best job on the board.

## 4. Going to work: the decision ladder  [PROPOSED, from David's example]

1. **My station** (if I'm assigned one): is there something to make? If it needs materials I don't have,
   solve the problem (below).
2. **The job board:** take the best open job I'm able to do. "Best" weighs how much I like it, my skill,
   how urgent it is, and how far away it is.
3. **Nothing to do:** idle (wander, chat). Being idle for long makes people unhappy.

## 5. Solving a problem: "I need wood"  [PROPOSED, from David's example]

Simple, obvious steps in order, each a little more effort than the last:

1. **My workshop's own storage.**
2. **The warehouse linked to my workshop** (linked in the manager).
3. **Any other warehouse** whose filters accept wood.
4. **Open hauling jobs carrying wood** (posted because of workshop settings): take one.
5. **Wood lying on the ground** where woodcutters or a tree farm are working, or anywhere known.
6. **Give up**, and do these:
   - leave the need on the board ("workshop 3 needs wood"). That can create work for others: a
     woodcutter sees a demand for logs;
   - say so where the player will see it (the manager, or a complaint);
   - go do something else, and try again later or **as soon as wood turns up** (a "wood stored"
     announcement wakes up everyone waiting for wood).

**Tools work the same way.** A dig job needs a pickaxe: my hands, my belt, my workshop, warehouses,
the ground, then give up ("no pickaxe anywhere").

**Reservations:** whoever claims a job, an item or a station **reserves** it, so two dwarves never walk to
the same log.

## 6. Happiness  [DECIDED]

- People have likes and dislikes among kinds of work. Doing jobs they like makes them happy; doing jobs
  they dislike, or having nothing to do, makes them unhappy.

## 7. Jobs at the three simulation levels  [PROPOSED]

- **Local:** the full thing. People walk, fetch, carry and work stations you can watch.
- **Rich** (your settlement while you're away): the job board keeps going, but jobs progress by summary
  math from how many workers there are, their skills, and what materials are available ("wall 40% built").
  The real result appears when you come back.
- **Thin:** no board, just production and growth numbers per trade.
- **Collapsing** keeps jobs in progress as records (who, what, how far along); **expanding** puts each
  worker back where their job is.

## 8. What gets saved  [PROPOSED]

- Each faction's **job board**: every job, its source (order / setting / opportunity), what it needs,
  urgency, who claimed it, how far along, and why it's stuck if it is.
- Each person's **assignment**, **current job and step**, likes and dislikes, and impulse levels.
- **Reservations are not saved**: they're worked out again from the claimed jobs when the world loads.
  That's simpler and can't go wrong.

## Questions

1. **Taking a hauling job (step 4):** is it a hauling job that would bring wood *to my own* workshop
   (so I go fetch it myself), or one taking wood somewhere else (so I redirect the wood to me, and the
   other place goes short)?
2. **Job boards:** one per settlement, or one per faction (a kingdom with several towns)?
3. **Disliked jobs:** do people still take them if nothing else needs doing (just less happy), or refuse
   them?
4. **Who tells you when someone gives up:** a line in the manager, a foreman's complaint, the person
   walking up to you if you're near?

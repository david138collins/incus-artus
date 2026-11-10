# Making the map  [FOR REVIEW, 2026-11-10]

Counsel's plan for the map David asked for: "a live document whiteboard that i look at as well as builder ... only
what exists and how its wired, so i could stare at it, for hours." Nothing here is decided; names and placement
are David's.

## 1. What the map is, and isn't

- **It is** the game as it stands today: each part, what it holds, what it reads and writes, when it runs, on the
  processor or the card, its cost as last measured, its tests, its files. And the wiring between the parts.
- **It isn't** history, quotes or plans. Those stay in the journals (`decisions.md`, `dyno.md`, `regroup.md`,
  `worldgen.md`, ...), linked from the map, never copied into it.
- **It's the first thing any reader reads:** David, Builder, Counsel, an Oracle, a new session.

## 2. Three zoom levels

**Level 0, the whole game on one screen.** The parts as boxes, the data flowing between them as arrows:

```
world generation ──► the world ──► sight ──► the screen
 (worldgen/)          (world/)     (sight/)    (runner/, ui, debug)
                         │            ▲
                         └── edits ───┘     the player, camera, kinematics
                                            sim, people, fate, time, events (parked)
```

**Level 1, a card for each part** (about 20: one per folder of `src/` that matters, the shaders, the data files):

| Field | Example (sight's store) |
|---|---|
| What it is | Two lines |
| What it holds | The cells, their bands, the pool, sizes in MB |
| Reads / writes | Who feeds it, who reads it |
| Runs when | At load, per tile, per frame, per edit |
| Where | Processor or card |
| Cost, last measured | ms or share of the card, with the build and date |
| Tests | Which ones guard it |
| Files | Paths |
| State | Liked / working / parked / under review |
| Open questions | One line each, linked |

**Level 2, the two pipelines step by step:**
- **The frame:** each pass in order (start, the layers, the march, listing, faces, shading, upscale, objects), what
  each reads and writes, its registers and share of the card.
- **The world's recipe:** its linear stages (maps, parts, uncarved ground, carving, cells), on which side
  (processor, card or both), and the to-the-bit checks between them.

**Across all levels:** the rules that bind (the projection principle, whole numbers, the linear pipeline, even beats
fast, nothing relies on fog, terrain light may lag, no per-card tricks): one line each, linked to `decisions.md`.

## 3. Its form

- **The source:** one file in the repo, `docs/map.md`, plain text with diagrams drawn in Mermaid (a text form of
  diagrams that VS Code, GitHub and Obsidian all draw). Every session can read and edit it; git keeps its history.
- **The view for David:** a page made from that file: the Level 0 diagram, a click on a box opens its card, a click
  on a pipeline step opens its detail. Private, opens on his phone as well. Republished whenever the file changes.
- **Option, David's call:** if he'd rather stare at it in Obsidian, the repo's `docs/` can be opened as a vault of
  its own (never his personal vault), where Mermaid draws and the graph view shows the links.

## 4. Keeping it true

- **Same commit:** whoever changes the wiring updates the map in that commit. Builder for the code's parts and
  costs; Counsel for the rules, the states and the page.
- **A check in the tests:** a test lists every folder of `src/` and every shader entry point (`@compute`,
  `@vertex`, `@fragment`) and fails if one isn't named in the map. The map can't fall silently behind the code.
- **Costs carry their build and date,** so a stale number shows as stale.

## 5. How it's built, step by step

Each step has a gate. Sizes are Builder's and Counsel's time plus David's looks.

1. **The skeleton** (Counsel, from Builder's review and the `src/` tree): Level 0 and the list of cards, empty.
   *Gate:* David reads it and says it's the right shape and the right names.
2. **The cards** (Builder fills each from the code; Counsel the rules and states). *Gate:* the check in section 4
   passes: every folder and every shader entry point named.
3. **The two pipelines** (Builder). *Gate:* Builder checks each step against the code; David can follow it.
4. **The page** (Counsel). *Gate:* David stares at it and says it works for him.
5. **The journals point at it:** a short "current state: see the map" line at the top of each.
6. **Then the Oracle** reads the map first, the journals after.

About an hour of Counsel's, two of Builder's, and three looks from David.

## 6. The workspace around it (to talk through)

- **The docs:** the map at the front; the journals behind it, unchanged, linked. Moving them into a folder of their
  own can wait: it breaks links for little gain.
- **The sessions:** Builder builds, Counsel keeps the decisions and the big picture, an Oracle reviews read-only at
  each big turn. Each reads the map at its start.
- **The screen:** the map's page open beside the chat, so a question about the engine starts from a box on it.
- **The tree:** two sessions in one working tree holds by discipline today. If a rebuild of sight starts, it may want
  a branch of its own (Builder's suggestion 5 in the review).
- **The ship list:** the smallest Incus Artus that delivers the joy, written down, so each part's "good enough" has
  a line to meet. Not part of the map, but the map makes it easier to write.

## 7. Questions for David

1. The name: "the map", or another?
2. Its look: boxes and arrows in the game's own palette, or plainer? (His eye: he'll be staring at it.)
3. Mermaid in the repo and a page for him, or Obsidian as well?
4. Start with step 1 now?

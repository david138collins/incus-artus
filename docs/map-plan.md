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

Drawn as swimlanes (Builder's note): the frame, loading and simulation lanes across, the processor and the card
down, so what runs when, and which way data crosses between processor and card, shows at a glance.

**Level 1, a card for each part** (about 20: one per folder of `src/` that matters, the shaders, the data files):

| Field | Example (sight's store) |
|---|---|
| What it is | Two lines |
| What it holds | The cells, their bands, the pool, sizes in MB |
| Reads / writes | Who feeds it, who reads it |
| Runs when | At load, per tile, per frame, per edit |
| Where | Processor or card |
| Lane | Frame, loading or simulation (several of today's costs were lane problems: the tile lane's pace, the old meshing on the frame's path) |
| Limits | The fixed sizes it lives within: the pool's 24M cells, the face table's 262,144 slots, the windows' 672 a side (hidden caps are where slowdowns hide) |
| Switches | Every `INCUS_*` setting and Render-page key that changes it: how we measure, and how we flip back |
| Keeps true | What it must never break (the processor's columns to the bit; every cell the exact sum of its children; the same picture as every block walked), each with the test that guards it |
| Cost, last measured | Card passes: share of the card on the dyno's route (whole / land / sea), ms, registers, groups in flight. Processor parts: ms a frame or a job. Each with its build, date and a link to its row in `dyno.md` |
| Tests | Which ones guard it |
| Files | Paths |
| State | Liked / working / parked / under review |
| Open questions | One line each, linked |

**Level 2, the two pipelines step by step:**
- **The frame:** each pass in order (start, the layers, the march, listing, faces, shading, upscale, objects), what
  each reads and writes, its registers, groups in flight and share of the card. **Each wait between passes marked**
  (about twelve a frame): the drains are where half the card's room goes, and they should be visible.
- **The world's recipe:** its linear stages (maps, parts, uncarved ground, carving, cells), on which side
  (processor, card or both), and the to-the-bit checks between them. **Its three copies side by side** (fractions
  for the editor's previews, whole numbers for the game's chunks, the card's for tiles and the bake) and where each
  is read, so the drift risk shows at a glance until it's settled (the review's route E).

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
- **A check in the tests, both ways** (Builder's shape): a plain test that reads `docs/map.md` and walks `src/`, run
  with the rest on every build (no card needed, like the doorway gauge). It fails if:
  - any folder of `src/`, top-level `.rs` file or shader entry point (`@compute`, `@vertex`, `@fragment`) isn't
    named in the map;
  - any `INCUS_*` variable the code reads isn't named (so every card's Switches are complete by construction);
  - any code name in the map (written `like_this`) no longer exists in the code (so dead names can't stay);
  - any cost line lacks its build and date.
  The map can't fall silently behind the code, or keep what's gone.
- **Costs carry their build and date,** so a stale number shows its age.

## 5. How it's built, step by step

Each step has a gate. Sizes are Builder's and Counsel's time plus David's looks.

1. **The skeleton** (Counsel, from Builder's review and the `src/` tree): Level 0 and the list of cards, empty.
   *Gate:* David reads it and says it's the right shape and the right names.
2. **The cards** (Builder fills each from the code; Counsel the rules and states). *Gate:* the check in section 4
   passes, both ways.
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

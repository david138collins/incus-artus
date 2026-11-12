# Starting fresh: a proposal  [FOR REVIEW, 2026-11-10]

David: "draft a proposal for packing up the old incus artus, and starting fresh; probably without bevy whatsoever
because it was just a bootstrap tool, really. fresh folder would include the packed up old incus artus and become our
working environment." And: "nothing that isnt ultimately part of the players joy is sacred to me."

Nothing here is decided. Whether to rewrite everything or only parts goes to the Oracle first; names and placement
are David's.

## 1. Why, and what must not be lost

**Why start fresh** (from Builder's review, `docs/review-2026-11-10.md`, and the dyno):
- The world's rules exist in three copies (fractions for the editor, whole numbers for the game, the card's copy).
- The recipe was never shaped for the card (255 registers, spilling; divisions by numbers known only as it runs).
- The near field is drawn by rays, where the card's triangles would do it for almost nothing.
- Light is worked out by rays every pixel, every frame, though terrain light hardly changes.
- The wiring is by hand, so no live map can be drawn from it; measuring came late.
- Bevy was a bootstrap: only its scheduler, states, time, input types and logging are left.
- The docs are journals, not maps.

**What must not be lost:**
- The world as David tuned it: tectonics, mountains, peaks, climate, rivers, lakes, caves, ores.
- The look he signed off: colours, blocky light, fog, far view, far lights.
- The editor's views (the World Compiler), the data files, the art, the presets worth keeping.
- The rules in `decisions.md`: the projection principle, whole numbers, the linear pipeline, even beats fast, nothing
  relies on fog, no per-card tricks, nothing temporary, terrain light may lag.
- The promises the tests guard: processor and card to the bit, every cell the sum of its blocks, the picture tests.
- The instruments: the dyno, the twin, the per-pass timers, the picture and bit tests.

**The danger** is the "second system": a rewrite tends to grow, and to lose the small tuned things that made the first
one right. Two guards: the spec decides what's in; and the old project is the check every new stage is tested against.

## 2. The new working folder

```
Incus Artus\                 the same path as today (see 5.4)
  master\                    what everyone looks at, David first
    map.md                   what exists and how it's wired
    plan.md                  the goals in order, as a nested checklist: what we're on, and why
    spec.md                  the game we're shipping
  journal\                   Counsel's notebook: every chronicle, with contents.md to find things
  old\                       Incus Artus as of build 109: its own git history, runnable, never edited
  src\  shaders\  assets\  data\  tests\     the new game
  CLAUDE.md                  every session reads master\ first
```

- **old\** keeps its own `.git`, so its 800-odd commits stay whole; the new project's git ignores it. It builds and runs
  from there with its own `target\` (never shared with the new one), so it can always be flown beside the new one.
- **journal\** takes today's `docs\` and `vision\` as they are, plus a contents page for Counsel. Nothing in it is
  rewritten; it's history.
- **master\** starts with this proposal's plan, the spec David cuts from Counsel's draft, and the map of the new
  structure as it's built.

## 3. What carries over, and how each is checked

| Part | How it comes over | Checked against old |
|---|---|---|
| Art, textures, models, palettes, block and object data | Copied as they are | — |
| Presets | Only the startup default, re-saved in the new format | The same world from it |
| The world's rules (recipe) | Written once, in whole numbers: one source for the processor, the card and the editor; cut into stages shaped for the card | The same seed gives the same columns, to the bit; or, if David allows a new formulation (see 7.4), side by side by eye at fixed places |
| Tectonics, climate, rivers, lakes, caves, ores | Rewritten from the known parameters, as linear stages | Their maps for the same seed, side by side |
| The editor (World Compiler) | Rebuilt on the one recipe, so it previews exactly what the game makes | David's use |
| The instruments | **First**, before anything they measure | — |
| The renderer | Designed after the Oracle's review (the hybrid near field, light kept in the world, a lean far march, or better) | Pictures at fixed places (the same spot, the same light) and the frame targets |
| The player, camera, movement, collision | Ported where small and liked | By feel |
| Simulation, people, fate, time, events | The toy rules ported when the spec reaches them | — |
| Bevy | Gone. Its scheduler and states replaced by our own small runner, built around the three lanes (frame, loading, simulation) | — |

## 4. The order, each step with a gate

0. **The Oracle** reviews the structure, the assumptions and the axioms, before the new structure is fixed. Its
   notes on Rust and wgpu come here. *Gate:* David's reading of it.
1. **The master documents:** the spec cut by David; the plan made from this proposal and the Oracle's answer.
   *Gate:* David signs both.
2. **Packing:** a dated backup to the USB drive (without `target\`); the tag `old-final` at build 109; the folders
   arranged as in section 2; old built and run from `old\`. *Gate:* old runs from its new place, and the backup
   reads back.
3. **The skeleton:** the runner and its lanes, the wiring declared as data, the map drawn from it, the control-room
   page in the game, the instruments and the tests. *Gate:* an empty frame measured, its map drawn from the
   declaration.
4. **The world's rules,** stage by stage. *Gate:* each stage matches old (to the bit, or by eye where David allows).
5. **The store and the bake.** *Gate:* their memory and time measured against old.
6. **The renderer,** by the Oracle's design. *Gate:* pictures against old at fixed places, then the frame targets.
7. **The editor.** *Gate:* David uses it.
8. **The game's systems,** in the spec's order.

Something to see early: a world map from the new recipe (step 4) can be shown before the renderer exists, so the
first weeks aren't blind.

## 5. Tools and workspace

1. **VS Code** (free), with the Claude Code extension and rust-analyzer. Nothing we do depends on IntelliJ; its
   link to these sessions goes with it.
2. **The sessions:** Builder, Counsel and the Oracle, each started in the new folder, each reading `master\` first.
3. **Off-site backup:** today's repository has no remote. `decisions.md` says "GitHub (private) for backup", but it was
   never set up; the USB drive is the only copy away from the laptop. A private GitHub repository for both projects
   is worth deciding before the packing (it's David's account to set up).
4. **The same path:** keeping `C:\Users\David\Desktop\Incus Artus` keeps Claude's project memory, David's desktop
   link and every habit pointing at the right place. The way there: close the sessions, rename today's folder, make
   the new one, move the old inside as `old\`, reopen the sessions. Counsel then prunes the memory notes that only
   apply to the old project into the journal.

## 6. Risks, and the guard for each

| Risk | Guard |
|---|---|
| The rewrite grows | The spec says what's in; the plan says what's next |
| Tuned subtleties lost | Old as the check, stage by stage |
| Weeks with nothing to see | The world map early; the renderer against old's pictures |
| Habits tied to old paths | `CLAUDE.md` rewritten; memory pruned |
| Elegance chased past need | "Match first, then refine": each stage first matches old, then gets simpler |

## 7. Decisions for David

1. Rewrite everything, or keep parts (the editor, world generation) and rewrite the rest? After the Oracle.
2. The same path with the old moved inside, or a new folder name?
3. The old folder's name: `old\`, or something else?
4. The recipe: to the bit against old, or may it be formulated anew where the result is "qualitatively identical"
   (your words of 2026-11-05: "it doesn't matter if worlds come out different, so long as the rules never changed")?
5. A private GitHub repository as the off-site backup?
6. VS Code?
7. When: after the Oracle's review?

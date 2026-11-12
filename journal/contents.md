# The journal: contents

How Incus Artus got here: proposals, assessments, experiments, and David's decisions in his own words. We add to it
and never rewrite it; a correction goes in the open, beside what it corrects. What's true now is in `master/`, one
folder up: read that first.

**About the paths inside.** These files were `docs/` and `vision/` at the top of the project until 2026-11-12, when
they moved here unchanged (plan 2.1). So:
- a path written inside them as `docs/…` or `vision/…` is now `journal/docs/…` or `journal/vision/…`;
- a code path (`src/…`, `assets/…`, `presets/…`, `Cargo.toml`, `build.rs`) is old Incus Artus, moved under `old/` the
  same day, and its line numbers are as they were then (`old-final` is the frozen code);
- `kept logs/`, `references/`, `exports/`, `notes/` and `Art/` stay at the top, where they were.

Kept by Counsel. Whoever adds a file here adds its line below.

---

## `docs/`: Builder's records, and the decisions

Until 2026-11-11 this was the record of what's decided and built; since then `master/` holds what's true now.

**The decisions**

| File | What it is |
|---|---|
| `decisions.md` | Every decision since 2026-10-21, in David's words where he gave them, tagged [DECIDED], [OPEN] or [LATER]. The first place to look for why something is the way it is. |

**The rebuild (2026-11-10 on)**

| File | What it is |
|---|---|
| `review-2026-11-10.md` | Builder's review of the whole software at build 109: what works, the graphics outlook, where to rework. The starting point for the Oracle. |
| `fresh-start-proposal.md` | The proposal for packing old away and starting fresh, which set out `master/`, `journal/` and `old/`. |
| `oracle-prompt.md` | The first message that called the Oracle, drafted for David to edit. |
| `map-plan.md` | Counsel's plan for `master/map.md`, the map of what exists and how it's wired. |
| `experiments.md` | The experiments flown in old (plan phase 1) and the pack's check flight: each pass mark written before it flew, then the results and the logs that hold them. |

**The engine, in old**

| File | What it is |
|---|---|
| `regroup.md` | Leaving Bevy and starting the graphics again on the card (2026-11-05), with the build-by-build numbers. |
| `far-view.md` | The far view as a ray marcher (2026-11-02 on), step by step. |
| `dyno.md` | The speed checklist (2026-11-09): every idea for making sight fast, each with its measured result. |
| `release.md` | What the player's copy leaves out or does differently from the development build (2026-10-26). |
| `data-files.md` | How old's plain-text data files work, and how they reload while the game runs. |

**The world, in old**

| File | What it is |
|---|---|
| `worldgen.md` | World generation from David's notes (2026-10-23 on): the Editor Suite, the World Compiler, and everything built since. The longest record. |
| `rivers.md` | Rivers from beginning to end, as built through 2026-10-30, with the shores reviewed. Its picture is `rivers-profiles-2026-10-30.png`. |
| `water-inventory.md` | The water code as it stood on 2026-10-31, part by part. |
| `water-rework.md` | Builder's answers to Counsel's eight questions on water (2026-10-31). |
| `caves.md` | Caves: what they touch, the options and an order (2026-11-01). |
| `quartile.md` | Quarter blocks: octants and bars, the options and the stops built (2026-10-31). |
| `placer.md` | The placer, its stops and what it has to grow into (2026-10-31). |
| `objects.md` | Real 3D objects in the world: torches, cups, the brazier (2026-11-01). |
| `block-properties.md` | What a block needs to know, from David's list (2026-10-21). |

**The game**

| File | What it is |
|---|---|
| `vision.md` | What the game is meant to become, from David's brief, his answers and his notes. |
| `simulation.md` | The simulation levels, the Goddess of Fate under every roll, and saving. |
| `jobs.md` | People, impulses and jobs, from David's notes (2026-10-22). |

---

## `vision/`: Counsel's

Counsel's folder since 2026-10-26: edited only when David asks. Its own `README.md` is the older index for it, written
when `docs/` was the record.

**Working files**

| File | What it is |
|---|---|
| `handoff.md` | Short notes between sessions. Read it at the start of a session; clear a note once it's acted on. |
| `oracle-review.md` | The Oracle's review (2026-11-11), and its second part, "before the first line". The one file the Oracle writes. |
| `oracle-return.md` | Counsel's brief for the Oracle's return at the fork (plan 2.3): what we've learned since its review, and our questions. Kept from 2026-11-11. |
| `watch-list.md` | Problems seen coming, and what would head each off (2026-10-26 on). |
| `projection.md` | The guiding principle (David, 2026-11-01): the form and its projection into blocks, "so that 1 always equals 1". |

**The bigger picture**

| File | What it is |
|---|---|
| `where-we-are.md` | Counsel's picture of the whole project (2026-10-26 on). Since 2026-11-11, `master/plan.md` keeps where we are. |
| `marketing.md` | The pitch and the path from no audience to launch (2026-10-26). |
| `how-its-made.md` | The facts and receipts behind "how was this made?", for David's own statement (2026-10-26). |

**Proposals and readings**

| File | What it is |
|---|---|
| `proposal-voxel-renderer.md` | Our own voxel renderer (2026-10-28), parked the same day, with Builder's assessment. |
| `proposal-peaks.md` | Peaks at the points (2026-10-27). |
| `toy-factory.md` | The FX Lab, once the Toy Factory (2026-10-26). |
| `terrain-controls.md` | Every World Compiler control and why David asked for it (2026-10-27, from old's code). |
| `terrain-pipeline.md` | What's calculated where in old's terrain, in order (2026-10-27). |
| `water-from-the-start.md` | Water regrouped from the start (2026-10-31), done enough to go on. |
| `ores-by-depth.md` | Ores and minerals of three games side by side, ranked by depth, for the mineralogy talk (2026-10-31). |
| `dyno-audit-2026-11-10.md` | An outside read of old's speed at build 104. |

**From before**

| File | What it is |
|---|---|
| `from-grok.md` | Ideas David gave Grok that no other file recorded, each waiting for his keep, change or drop (2026-10-26). |
| `incus_artus_full_conversation.txt` | The Grok conversation, kept as a record. |
| `README.md` | The folder's own index (2026-10-26), older than this page. |

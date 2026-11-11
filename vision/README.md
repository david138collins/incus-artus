# vision/ — the project counsel's folder

This folder is where the **big picture** lives: where Incus Artus is, where it's going, what could go
wrong, and how it becomes a product that sells. It's kept by Claude acting as David's vision manager and
general counsel (the role Grok held until 2026-10-26), in conversations that step back from the code.

`docs/` stays the source of truth for **what's decided and built**. This folder never overrides it.
If the two disagree, `docs/` wins, and the disagreement goes on the watch list to be settled.

## Files

| File | What it's for |
|---|---|
| `projection.md` | **The guiding principle** (David, 2026-11-01): the metaphysical world (the form) and the physical world (its projection into blocks), "so that 1 always equals 1". Read before designing any feature. |
| `where-we-are.md` | The whole scope in brief, what exists, the road ahead, and the road to selling it. Updated after each counsel session. |
| `watch-list.md` | Problems seen coming before they arrive: risks, conflicts between plans, decisions that get expensive if left late. |
| `marketing.md` | The pitch, the moments people will share, and the path from no audience to launch. |
| `proposal-voxel-renderer.md` | Proposal: our own voxel renderer inside Bevy, and everything it must draw, before trees arrive. |
| `toy-factory.md` | Proposal: growing the Graphics Kit into the place the game's small reactive effects are made. |
| `terrain-controls.md` | Every World Compiler control, what it does, and why David asked for it. For the controls rework. |
| `terrain-pipeline.md` | What's calculated where in the terrain, in order, and where a new layer could go. |
| `how-its-made.md` | The facts and receipts behind "how was this made?": who does what, what proves it, and how receipts are kept. |
| `handoff.md` | Notes between the two Claudes (IDE and counsel), since neither can see the other's conversation. Both read it at the start of a session. |
| `from-grok.md` | Ideas from the Grok conversation that never made it into `docs/`. Each waits for David to confirm before it's promoted. |
| `incus_artus_full_conversation.txt` | The Grok conversation, rendered by Grok (David's messages in full, most of Grok's replies left out). Kept as a record. |

## For Claude in the IDE

You build what David asks for next; this folder says why and where it's heading. Read it when:

- a task touches something on the **watch list** (say so before building, don't silently work around it);
- a choice now would make a later system harder (the Fate service, saving, the simulation levels, the demo);
- David asks "what's next?" or "does this fit?".

Don't edit these files unless David asks. If something you build changes the picture (a risk is gone, a
plan turned out wrong), mention it to David so it gets carried back here.

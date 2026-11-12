# How Incus Artus is made: the facts and the receipts

An outline for David to write the public statement from (Steam page, website, devlogs). The words of
that statement should be David's own. This file only gathers what's true and what can be shown, so
that when someone honestly asks "is this just something an AI spun up?", the answer is ready and provable.
Started 2026-10-26.

---

## The facts

**Who does what**
- **All the art is David's, drawn by hand.** Textures, models, animations, UI, map stamps. Drawn in
  Aseprite and Grafx2, in palettes chosen colour by colour. No AI-generated images, ever.
- **All the design is David's.** The world, its rules and systems, the story, the art direction, and all
  the tuning: every slider in the game exists so David can set it by hand.
- **The code is written by an AI coding agent (Claude), to David's design.** David doesn't write the code.
  He reads it, questions it, has it explained, and decides what it does. He set this rule for himself:
  no decision and no art is ever outsourced.

**How it started**
- A lifelong player of Minecraft and Dwarf Fortress who wished one game could be both.
- About two months each spent learning C, Python and Rust, which was enough to understand code but not
  to write a game.
- A Minecraft mod first (with Junie in IntelliJ), then the move to a standalone engine in Rust and Bevy
  in October 2026.

**How much of it is David's thinking** (figures as of 2026-10-26; update before publishing)
- Over 150,000 words written by David working on the game (David's count).
- 42 places in the design documents are marked as David's own idea or call, and that undercounts:
  most decisions aren't marked at all.
- Around 23,000 words of design documents, all of them recording what David decided.

---

## The receipts

What exists, where, and whether it's safe:

| Receipt | Where | Kept safe? |
|---|---|---|
| **Development history:** every change, dated, with what David asked for | the project's git history (89 commits since 2026-10-21) | Yes, in git |
| **Design documents:** every decision, dated, with David's name on his calls | `docs/` | Yes, in git |
| **The vision:** the big picture, the plans, this file | `vision/` | **Not yet** (uncommitted) |
| **The Grok conversation:** months of design talk, from the mod days on | `vision/incus_artus_full_conversation.txt` | **Not yet** (uncommitted) |
| **Session transcripts:** David's own messages from Claude sessions | `Art/session 2026-10-24 - David's messages.txt` (one so far) | **No** (untracked) |
| **Art source files:** layered `.aseprite` files, not just the exported PNGs | `Art/` | **No** (untracked) |
| **Palettes:** every `.gpl` chosen by hand | `palettes/` | **No** (untracked) |
| **Tuning presets:** David's slider settings | `presets/` | **No** (untracked) |
| **The notes vault:** David's lore, lists and language | `lorecraft/` (outside the project) | Unknown |
| **The Minecraft mod:** the earlier project | outside this project | Unknown |

The weak spot is clear: **the code's history is kept perfectly, and the art and design history mostly
isn't.** For this project, those are the receipts that matter most.

---

## Keeping receipts from now on

1. **Commit the art, palettes and presets.** They're the proof the art is hand-made: `.aseprite` files keep
   layers and frames, and git dates every version. (Large binary files can be handled with Git LFS if the
   repository grows heavy.)
2. **Export David's messages from Claude sessions** now and then, as on 2026-10-24, and keep them with the project.
3. **Keep the Grok conversation and the notes vault** backed up alongside the project.
4. **Record some art sessions** (OBS): a drawing timelapse is the most convincing receipt there is, and
   good devlog content too. David works Aseprite "like a flight simulator" with his own keybindings, so
   recordings will show real, fluent hand work.
5. **Keep writing David's name next to his ideas** in the docs, as both Claudes already do.
6. **Back up off this computer.** Receipts on one hard drive aren't receipts. A private online
   repository (GitHub, for example) keeps the full history safe and dated.

---

## When it's published

- **Say it first,** in the first devlog and on the Steam page, before anyone asks.
- **Steam's AI disclosure:** check the current rules when the page goes up and answer them fully.
  The honest answer is short: code written by an AI agent to the developer's design; no AI-generated art,
  sound or writing in the game. (Keep that last part true: if AI is ever used for anything in the game
  itself, the statement must change.)
- **Show, don't argue:** a page of excerpts from the design documents, a drawing timelapse, and the
  history graph speak for themselves. People who've already decided won't be moved; people who honestly
  ask will be.

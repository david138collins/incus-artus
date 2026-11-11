# Incus Artus

An open-world survival and settlement simulation: Minecraft-style building and exploring with Dwarf
Fortress-style living settlements. Rust and Bevy. David designs, makes the art and tunes; Claude writes
all the code.

## Where things are written down

- `docs/` is the source of truth for what's **decided and built**. Keep it up to date as you work.
- `vision/` is the **big picture**, kept by a separate Claude session acting as David's vision manager
  (the role Grok used to have). Start with `vision/README.md`. Don't edit these files unless David asks.
- `vision/handoff.md` is how the two Claudes leave each other notes. **Read it at the start of a
  session**, and add a note when something you built changes the big picture.

## Git

Both Claudes work in this one folder. Work on `main` unless David says otherwise, and never switch
branches without checking `git status` and telling David first: a switch moves the other session too.

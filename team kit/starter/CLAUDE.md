# <Project>

<One or two lines: what it is, and what it's made with.> David designs, makes the art and tunes; Claude writes all
the code.

## Start here

`master/` is the living truth. Read it first, starting with "Where we are" in `master/plan.md`:

- `master/plan.md`: where we've decided to go, in order, and where we are on the way.
- `master/spec.md`: the thing we're shipping (David's to cut).
- `master/map.md`: what exists and how it's wired, drawn from the code itself.

## Who's who

- **David** holds the vision, makes the calls, the art and the tuning.
- **Builder** writes all the code, and ticks the plan with the commit or log that proved each line.
- **Counsel** keeps the plan and the decisions with David, and carries messages between him and Builder.
- **The Oracle** reads and never edits (except `journal/vision/oracle-review.md`); it's called at the big forks.

## Where things are kept

- **`master/`:** what's true now (above). Keep it true as you work.
- **The journal:** `journal/`, found through `journal/contents.md`. How we got here, and David's decisions in his
  own words (`journal/docs/decisions.md`). Add to it; never rewrite it. `journal/vision/` is Counsel's: don't edit
  it unless David asks.
- **`journal/vision/handoff.md`:** short notes between sessions. Read it at the start of a session; clear a note
  once it's acted on.
- **Memory** (one folder Builder and Counsel share; its index is loaded into every session): only habits and
  warnings about working with David and this project. Never the plan, the project's state or numbers: those go
  in `master/`.
- **Session transcripts** aren't a store: they're summarised and forget.

## Git

Builder and Counsel work in this one folder; the Oracle only reads. Work on `main` unless David says otherwise,
and never switch branches without checking `git status` and telling David first: a switch moves the other
session too. Commit your own files by name (`-- path`), never `git add -A`: the index is shared.

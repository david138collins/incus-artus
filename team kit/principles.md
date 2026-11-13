# Principles

The ones that held up. Each is a rule of thumb with its reason, not a law; question any of them when a project's
facts say so, and say which one and why.

## Building

- **Linear, modular, hierarchical.** Work flows one way through stages, each reusable, swappable and re-tunable.
  Never feed a later stage's output back into an earlier one. Say plainly which stage a feature lives in and what it
  reads from upstream. Numbers live in data, one formula per behaviour. Why: David re-tunes early stages in the light
  of later ones, and plans a full revision pass at the end; tangled stages make both painful.
- **Shared pieces once.** One palette, one colour system, one recipe that everything reads; never copies that can
  drift apart.
- **Carry results, not code.** When rebuilding, check the new against the old's results (exact data, fixed-view
  pictures, measured flights), not against its source. Old code is copied only where David or the look says it's right.
- **Nothing temporary.** Build what the user will see, from the start; no throwaway stand-ins, because stand-ins
  become part of the thing. Instruments and test harnesses are fine. Nothing visible is built until it's been talked
  through with David and he says go.
- **The right result in the middle of the knob.** Settings start wide enough to explore and narrow around where the
  good results are. If good results keep needing a slider pinned at one end, its range or its meaning is wrong.
- **Throwaway snapshots get no migration code.** Presets and tuning saves aren't worth shims; keep only the default
  loading.
- **Decide early what changes how art is made,** and write the art rules down, so no art is made wrong.

## Measuring

- **Measure from the start.** Every new piece is timed on its own from its first build, and the map shows its cost.
  Measuring early is not optimising early.
- **Look first, then the dyno.** Get it looking right and fully featured, then inspect how it uses the hardware, then
  add power one measured step at a time. Even choppiness is far less offensive than irregular lag.
- **Spend headroom gradually.** Each step that adds cost is talked through first (what it does, how, what it should
  cost), built alone, measured alone, and its price written down before the next goes on. Never bundle visual steps.
- **A pass mark before every flight,** written in the journal: what good is, the estimate marked as an estimate, what
  counts as a miss. Then the result beside it, wrong guesses included.
- **One change at a time,** each behind its own switch, kept only if the numbers and David's eye say so.
- **Code-reading guesses about speed are often wrong;** every speed idea is priced with a measurement before anyone
  believes it. An outside audit is welcome once the plan stops changing every flight, and its findings are measured too.
- **Know the noise.** A repeat flight can differ by a percent or two; a warm-up can make the first half minute of a
  session slower. Measure settled, and read one flight with its noise.

- **No megakernels.** Build card work as small passes over shared per-pixel stores, each timed on its own, not one
  giant program: in a register-bound giant, code added costs every pixel even when switched off, and the compiler's
  layout becomes a lottery. Split one piece at a time and measure each split alone; splitting isn't always faster.
  "Small" means small in code, not many launches a frame: launches in a row add their waits, and one launch covering
  several kinds of pixel can overlap waits that separate launches can't.
- **Read the watts beside the load.** A card can report itself fully loaded while it waits on a few long chains of
  work; its power draw falling as the frame gets dearer says it's queued, not working.
- **Ask idle hardware what it can do.** If any part of the ordinary hardware sits underused (a fixed-function unit,
  free memory, the time between frames, spare threads), ask "what can *you* do for me?" Keep what the program needs to
  run within the smallest machine's budget; let what only makes it faster or finer grow into what a bigger machine
  has free, and drop away on a small one, giving the same result either way. Portable features only, never one
  vendor's trick.

## Keeping it true

- **Corrections in the open.** A wrong figure, a miscount, a bad read: corrected beside the original, saying what was
  wrong and what's right, never silently.
- **The map is drawn from the code.** What exists and how it's wired is declared once, in the code, and the docs are
  drawn from it by the build's tests, so they can't fall behind.
- **Diagnostics first.** The program writes its own crash, freeze and session reports from early on. On any crash or
  freeze, read them before guessing.
- **The truth is in files.** `master/` for what's true now, the journal for how we got here, memory for habits. A
  session can read its way to the present without anyone's recollection.

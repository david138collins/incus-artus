# Habits and warnings

Practical things learned the hard way. Each has its reason, so a later session can tell when it no longer applies.

## Git, with two sessions in one folder

- **Commit your own files by name, with a pathspec:** `git commit -m "..." -- path/one path/two`. Never `git add -A`,
  `git add .` or a whole folder. Why: the index is shared, so a plain commit takes whatever the other session has
  staged; and David keeps some edits of his own uncommitted on purpose.
- **Look before committing:** `git status --short` and `git diff --cached --name-only`.
- **Committing only your part of a file someone else has also changed:** build the staged copy from the working file
  with *their* hunks reversed, then stage it directly:
  1. `git diff -U0 <file> > all.patch`; copy the header and their hunks into `others.patch`.
  2. Copy the file into a temporary folder at the same relative path; there, `git apply -R --unidiff-zero others.patch`.
  3. Check: `diff --strip-trailing-cr <(git show HEAD:<file>) <temp copy>` shows only your changes.
  4. `blob=$(git hash-object -w --path=<file> <temp copy>)`, then `git update-index --cacheinfo 100644,$blob,<file>`.
  Why: keeping your own hunks with `git apply --cached` misplaced text once, when the other session's hunks shifted
  the line numbers.
- **A long commit message** goes in a file in the scratchpad, then `git commit -F <file> -- paths`. In PowerShell a
  here-string piped to `-F -` can be read as pathspecs.
- **Push when it suits;** David has said push whenever. Never push someone else's branch.
- **A worktree or an older checkout gets its own build folder.** Sharing one let an old program's exe stand in for the
  new, and David was launched into the wrong version.

## Windows

- **Edit text with the Edit and Write tools, never PowerShell's `Set-Content`:** Windows PowerShell 5.1 reads UTF-8
  without a BOM as Windows-1252 and mangles dashes, arrows and other symbols. If a shell rewrite is unavoidable, use
  .NET `ReadAllText`/`WriteAllText` with UTF-8.
- **Bash heredocs mangle backslashes and triple quotes:** write long scripts to the scratchpad with Write first.
- Python with `-I` when it reads downloaded or traced data.

## Rust and builds

- **Launch with `cargo run --release`** from the project's own folder, never the exe directly: without Cargo's
  folder the program can't find its shaders and data. Never the dev profile for anything David plays or measures.
- **Don't run a formatter over a hand-styled codebase** without a config file for it: it rewrites every file and
  buries the real changes. Match the surrounding style.
- **Number the builds:** a `BUILD` file at the top, bumped by one in the commit before each launch handed to David,
  shown somewhere out of sight in the developer panel. A number never names two programs.
- **Don't compile while David plays,** or during a flight or a test window; never edit code while a script of
  flights is pending (each run rebuilds).
- **A flight's report says which commit it flew and whether the code was committed;** read that line before
  believing the numbers.

## The card's work

- One big piece of card work holds the frame: background work goes in small pieces, waited for gently.
- In a register-bound shader, code added costs every pixel even when it's switched off: price every change by flight.
- Watch for card-hostile arithmetic: division and remainder by values known only at run time, local arrays.
- Never take a slope as the difference of two nearby rounded values.
- The graphics library's safety checks can cost a third of the card or more: price each program checked and unchecked.

## The laptop

- An Acer Predator with two cards: NVIDIA for play and NVIDIA flights ("NVIDIA GPU only" in PredatorSense, Personal
  Settings, "GPU Operating"); the Intel card's flights need "Optimus", so it drives the screen itself. Never
  "Automatic Selection" for measuring. Switching needs a restart, which ends the sessions: commit and write notes first.
- DirectX 12, not Vulkan: on this laptop Vulkan once stalled at 6 frames a second while DX12 ran at 240.
- It runs in Turbo with the fans at full as normal, and touches 100 °C routinely. Read watts, clocks and frame rate
  for heat questions, not degrees. Don't ask him to change that mode without a reason.
- On a freeze or a blue screen, check the graphics driver's events and version first, before blaming the code.
- Registers and occupancy show only in Nsight's own window: ask David to read them.

## Backups

- A private GitHub repository from day one, pushed as work lands.
- A dated copy on David's USB drive at milestones, offered once, never nagged: `robocopy <project> <dated folder> /E
  /XD <project>\target` (exit 1 is success). Check the drive letter first, and never touch his music folders on it.
- What isn't in git (art, notes, untracked files) has no other copy until it's on the USB.

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
- **Hold your commits while the other session is flying.** Each flight runs `cargo run` afresh, so a commit made
  mid-script changes the build's stamp: the next run rebuilds, and its report names a different commit. The code
  may be the same, but the reports no longer line up at a glance. Ask to be told when a script of flights starts
  and ends.
- **A worktree or an older checkout gets its own build folder.** Sharing one let an old program's exe stand in for the
  new, and David was launched into the wrong version.

## Measuring and checking

- **Never measure in a program's first half minute.** Its first 25 seconds or so can run a fifth slower, the card
  included, while clocks come up and the window settles. Warm up before the first measure; a floor once taken inside
  the warm-up was wrong and had to be corrected in the open.
- **Prove a check bites.** When an exact comparison passes, break one input on purpose (one colour, one step) and
  watch it fail, then put it back. A perfect match is only worth something if a wrong one would show.
- **Test the edges, not just the middle:** the first and the last, at every level. A test that covered only the
  middle passed, and the first launch crashed on the last piece of a row at the edge.
- **Look at the picture yourself before handing it over.** Draw it off the screen in a test, save it and look;
  compare before and after side by side. It's cheaper than a launch, and it catches what numbers don't.
- **Medians don't add.** The passes' medians don't sum to the frame's; read the whole from its own clock.
- **Compare within one session, alternating.** Two builds flown back to back, A B A B, agreed to about 1%; the same
  builds compared across sessions (other heat, other runs) differed by up to about 5%, and once showed a 7.5% "cost"
  that wasn't there. Small differences are only real inside one session.

## Windows and tools

- **Edit text with the Edit and Write tools, never PowerShell's `Set-Content`:** Windows PowerShell 5.1 reads UTF-8
  without a BOM as Windows-1252 and mangles dashes, arrows and other symbols. If a shell rewrite is unavoidable, use
  .NET `ReadAllText`/`WriteAllText` with UTF-8.
- **Bash heredocs mangle backslashes and triple quotes:** write long scripts to the scratchpad with Write first.
- **Edit scripts in Python use raw strings** (`r"""…"""`): in a plain string `\n` becomes a real line break and the
  match fails. A script that stops before writing anything is safe, but slow.
- **Never `rm` through a shell variable** (`rm -f "$P"/*`): the safety check refuses the whole command, and nothing
  in it runs. Use a literal path, or `"${P:?}"`.
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
- **Taking the graphics library's safety checks off isn't a sure win.** Once it made light views faster but the heavy
  views of a big shader a third to a half *slower*, likely the compiler laying out its long loops worse. Price each
  program checked and unchecked; don't assume either.
- Ask the card for the library's default limits, and raise one only on purpose, saying so in the code and the log
  (the largest buffer, say, raised for a whole world kept on the card).
- **Stay at the default limits for what must run.** A raised limit is a portability risk: an older or smaller card
  may not allow it, and the program then fails to start on the very machine named as the floor. If there's a cheap
  way to stay at the default, take it; ask later what the raised limit could buy as an optional extra, with the
  default way kept as the fallback.

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

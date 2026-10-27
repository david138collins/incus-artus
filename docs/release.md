# The player's copy: what a release build changes

What the game Steam installs will leave out or do differently from the development build, written
down 2026-10-26 so it's there when release planning starts. Nothing here changes the game's design.
Marked **[BUILT]** where it's already done; the rest is **[PLAN]**.

## 1. The build

- **[BUILT] The dev switch.** The developer tools are a Cargo feature, `dev`, on by default. A player's
  copy is built with `cargo build --release --no-default-features`: no debug panel (F3), no Editor
  Suite (F2), no self-tests, soak test, screenshot key or temperature readout, and block files aren't
  reloaded as they're edited. Their code is still compiled, just never switched on; a tool added from
  now on should be switched on under `#[cfg(feature = "dev")]` too.
- **[PLAN] Full optimisation:** link-time optimisation, one code unit at a time, debug information
  stripped. A little faster, and a smaller download.
- **[PLAN] Bevy trimmed** to what the game uses (it builds everything by default: audio formats,
  3D model loading, animation and more).
- **[PLAN] A baseline CPU** for the build, chosen with the minimum spec.

## 2. Left out of the download

- Only the program and `assets/` (block data, textures, shaders) ship.
- Not `Art/`, `docs/`, `vision/`, `world bakes/`, `presets/`, `screenshots/`, `logs/` or `exports/`.
- **[PLAN]** Textures in a compressed format the graphics card reads directly (smaller, faster to load).
- **[PLAN]** Logs keep only warnings and errors; the event log isn't written to disk by default.

## 3. How it behaves on the player's machine

- **[BUILT] A frame cap** (Render page): a frame every refresh, every 2nd, 3rd or 4th, or uncapped.
  Default every 2nd refresh. The biggest single saving in heat and power: drawing 240 frames a
  second for a 60 Hz screen wastes three quarters of the work. A player's options menu would offer
  the same.
- **[PLAN] Leave cores free.** Background work (making chunks, meshing, distant tiles) uses a set
  number of threads instead of one per core, so Windows and anything else running keep some.
- **[PLAN] Rest when not in front:** stop drawing when minimised, slow right down in the background.
- **[PLAN] Graphics presets** (Low, Medium, High), picked to suit the machine: render distance,
  distant terrain rings, light map detail, near lighting detail.

## 3½. When something goes wrong

- **[BUILT 2026-10-27] Diagnostics** (`src/diagnostics.rs`), on in a player's copy too, and costing nothing
  a frame. In `logs/`: a **session log** (everything the game logs), **crash reports** (the message, the
  file and line, the chain of calls) and **freeze reports** (a watchdog notices no frame for 5 seconds, and
  says when the game comes back). Each report says what was going on: the screen, the debug page, the
  Editor Suite tab, the last key presses and clicks, the session log's last lines, and the build (commit,
  "+ changes" if uncommitted). A `running.txt` marker shows the next start whether the last session ended
  normally; if not, it says so on screen and on the debug panel's Overview page. Release builds keep line
  tables (`[profile.release] debug`) so reports name files and lines. `INCUS_TEST_FAULT=crash` / `=freeze`
  checks the reports.
- **[PLAN]** For a player's copy: where `logs/` lives (the user's data folder, not next to the program), a
  way to send a report, and whether to strip line tables from the download. Full Windows crash dumps
  (every thread's state, for crashes inside graphics drivers) if the reports ever fall short.

## 4. Memory

- **[PLAN]** A compiled world keeps ten full-size maps (about 200 MB for a 2,250-pixel bake); most
  are only needed to make the world or tune it in the Editor Suite. A player's game keeps what the
  terrain is made from, at lower precision where that's enough. (The development build holds about
  1 GB while flying; the soak test measures it.)
- **[PLAN]** Chunk save files compressed; saving in the background.

## 5. The target spec

The Performance page's budget: 240 FPS on David's laptop, standing in for a GTX 980 Ti at 1080p on low
settings. Minimum and recommended specs are decided nearer release; sections 1, 3 and 4 are tuned to
meet them. The soak test (`src/soak.rs`) checks the game doesn't run worse the longer it runs.

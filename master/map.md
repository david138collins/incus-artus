# The software map

What exists and how it's wired, drawn from the code's own list of its parts (`src/wiring.rs`) by its test on
every build: change the list, never this file. In the game, the same map is live: the editor (F2), "Software map".

The words: a **lane** is a line of work with its own pace; a **pass** is one piece of a lane's work, on the
processor or the card; a **stage** is one step of the world's recipe (none yet); a **store** is something held.

The costs are the newest kept flight's, at its first standing, `kept logs/flight-1794489087.txt`: Incus Artus, build 119 (b48ddc3); the code committed.

```mermaid
flowchart TB
  subgraph lane_the_window["the window (as the window's events come)"]
    direction LR
    pass_hearing_the_window["hearing the window"]
  end
  subgraph lane_the_frame["the frame (once a frame)"]
    direction LR
    pass_going_to_a_mark["going to a mark"]
    pass_flying["flying"]
    pass_laying_out_the_panel["laying out the panel"]
    pass_getting_the_picture["getting the picture"]
    pass_the_panel["the panel"]
    pass_handing_the_card_its_work["handing the card its work"]
    pass_showing_it["showing it"]
    pass_measuring["measuring"]
  end
  subgraph lane_loading["loading (as tiles are asked for)"]
    direction LR
    pass_the_doorway["the doorway"]
  end
  store_the_keys_held[("the keys held")]
  store_the_camera[("the camera")]
  store_the_clocks[("the clocks")]
  store_the_wiring[("the wiring")]
  store_the_panel_s_shapes[("the panel's shapes")]
  store_the_screen_s_picture[("the screen's picture")]
  store_the_flight_s_report[("the flight's report")]
  store_the_session_s_log[("the session's log")]
  store_the_export[("the export")]
  store_a_tile_s_columns[("a tile's columns")]
  store_the_marks[("the marks")]
  store_what_changed[("what changed")]
  pass_hearing_the_window --> store_the_keys_held
  pass_hearing_the_window --> store_the_camera
  store_the_marks --> pass_going_to_a_mark
  store_a_tile_s_columns --> pass_going_to_a_mark
  pass_going_to_a_mark --> store_the_camera
  pass_going_to_a_mark --> store_what_changed
  store_the_keys_held --> pass_flying
  pass_flying --> store_the_camera
  store_the_camera --> pass_laying_out_the_panel
  store_the_clocks --> pass_laying_out_the_panel
  store_the_wiring --> pass_laying_out_the_panel
  pass_laying_out_the_panel --> store_the_panel_s_shapes
  store_the_panel_s_shapes --> pass_the_panel
  pass_the_panel --> store_the_screen_s_picture
  store_the_screen_s_picture --> pass_showing_it
  pass_measuring --> store_the_clocks
  pass_measuring --> store_the_flight_s_report
  store_the_export --> pass_the_doorway
  store_what_changed --> pass_the_doorway
  pass_the_doorway --> store_a_tile_s_columns
```

## The lanes

### the window

- **Runs:** as the window's events come.
- **What it is:** What the window hears: keys pressed and let go, the mouse moving, the window resized, closed or left.
- **Its passes, in order:** hearing the window.
- **Files:** src/runner.rs.

### the frame

- **Runs:** once a frame.
- **What it is:** Each frame, its passes in order: the camera goes to a mark if it's time, flies, the panels are laid out, the card draws the picture, and it's handed to the screen.
- **Its passes, in order:** going to a mark, flying, laying out the panel, getting the picture, the panel, handing the card its work, showing it and measuring.
- **Switch** `INCUS_FLIGHT`: a measured flight: every mark in data/marks.toml in turn, 5 s settling and 20 s measured at each, uncapped with the panel hidden; a report, then quits.
- **Switch** `WGPU_ADAPTER_NAME`: the card, by part of its name (intel: the laptop's Intel card).
- **Switch** `WGPU_POWER_PREF`: the card, by power (low or high).
- **Switch** `WGPU_BACKEND`: the card's language (DirectX 12 unless set).
- **Switch** `WGPU_DX12_COMPILER`: the shader compiler (the Windows SDK's DXC unless set).
- **Files:** src/main.rs, src/runner.rs and src/card.rs.

### loading

- **Runs:** as tiles are asked for.
- **What it is:** The world, read a tile at a time through the doorway. Going to a mark asks it for the ground under the eye; the far cells' lane will ask for tiles (2.2).
- **Its passes, in order:** the doorway.
- **Files:** src/doorway.rs.

## The passes

### hearing the window

- **A pass** in the window, on the processor.
- **What it is:** Each event: the keys held kept, the mouse turning the camera while it's captured, and F2 (the editor), F3 (the panel), F11 (window or full screen) and Tab (the mouse).
- **Reads:** none. **Writes:** the keys held and the camera.
- **Cost, last measured** (`kept logs/flight-1794489087.txt`): on the processor not run.
- **Files:** src/runner.rs and src/camera.rs.

### going to a mark

- **A pass** in the frame, on the processor.
- **What it is:** At the start, and on a flight once a mark is measured: the camera taken to the next mark, a player's eye over the ground the doorway answers, and the mark's changes made through the doorway (the build's placings, the scatter).
- **Reads:** the marks and a tile's columns. **Writes:** the camera and what changed.
- **Cost, last measured** (`kept logs/flight-1794489087.txt`): on the processor median 0.000, 95th 0.000, 99th 0.000, worst 0.199.
- **Files:** src/marks.rs and src/runner.rs.

### flying

- **A pass** in the frame, on the processor.
- **What it is:** The camera flies by the keys held (WASD, Space, Ctrl, Shift), on real time. Not while the editor is open.
- **Reads:** the keys held. **Writes:** the camera.
- **Cost, last measured** (`kept logs/flight-1794489087.txt`): on the processor median 0.000, 95th 0.000, 99th 0.001, worst 0.343.
- **Files:** src/camera.rs.

### laying out the panel

- **A pass** in the frame, on the processor.
- **What it is:** The developer panel (F3) and the editor (F2) laid out, and cut into shapes for the card.
- **Reads:** the camera, the clocks and the wiring. **Writes:** the panel's shapes.
- **Cost, last measured** (`kept logs/flight-1794489087.txt`): on the processor median 0.008, 95th 0.090, 99th 0.166, worst 0.403.
- **Files:** src/panel.rs, src/editor/mod.rs and src/editor/software_map.rs.

### getting the picture

- **A pass** in the frame, on the processor.
- **What it is:** The screen's next picture to draw on, waited for if none is free (capped, this is where the frame waits for the screen).
- **Reads:** none. **Writes:** none.
- **Cost, last measured** (`kept logs/flight-1794489087.txt`): on the processor median 0.004, 95th 0.160, 99th 0.336, worst 0.653.
- **Files:** src/runner.rs.

### the panel

- **A pass** in the frame, on the card.
- **What it is:** The picture cleared to the sky's colour, and the panels drawn over it. Recorded on the processor, run on the card.
- **Reads:** the panel's shapes. **Writes:** the screen's picture.
- **Cost, last measured** (`kept logs/flight-1794489087.txt`): recording it on the processor median 0.004, 95th 0.158, 99th 0.177, worst 1.634; on the card median 0.018, 95th 0.023, 99th 0.041, worst 0.300.
- **Files:** src/runner.rs.

### handing the card its work

- **A pass** in the frame, on the processor.
- **What it is:** The frame's work for the card finished and handed to it, with the card's clock asked to copy its times out.
- **Reads:** none. **Writes:** none.
- **Cost, last measured** (`kept logs/flight-1794489087.txt`): on the processor median 0.076, 95th 0.241, 99th 0.319, worst 1.644.
- **Files:** src/runner.rs.

### showing it

- **A pass** in the frame, on the processor.
- **What it is:** The picture handed to the screen, waited for if the screen's queue is full (Windows lets the frame run only so far ahead of what the card and the screen have taken).
- **Reads:** the screen's picture. **Writes:** none.
- **Cost, last measured** (`kept logs/flight-1794489087.txt`): on the processor median 0.142, 95th 0.271, 99th 0.329, worst 1.755.
- **Files:** src/runner.rs.

### measuring

- **A pass** in the frame, on the processor.
- **What it is:** The frame's length, and the card's times read back from frames it has finished; on a flight, kept for its report.
- **Reads:** none. **Writes:** the clocks and the flight's report.
- **Cost, last measured** (`kept logs/flight-1794489087.txt`): on the processor median 0.009, 95th 0.017, 99th 0.090, worst 0.472.
- **Files:** src/runner.rs, src/clocks.rs and src/flight.rs.

### the doorway

- **A pass** in loading, on the processor.
- **What it is:** Answers "what's here?" a tile at a time: each column's ground, its river's or lake's water, and its blocks from bedrock up as runs; and what changed in it. The engine never opens a world file itself. The export is read-only.
- **Reads:** the export and what changed. **Writes:** a tile's columns.
- **Cost, last measured** (`kept logs/flight-1794489087.txt`): on the processor not run.
- **Files:** src/doorway.rs.

## The stores

### the keys held

- **A store** on the processor.
- **What it is:** The keys held down now (let go of all when the window is left).
- **Written by:** hearing the window. **Read by:** flying.
- **Files:** src/runner.rs.

### the camera

- **A store** on the processor.
- **What it is:** Where the eye is (north, east, up, in blocks) and where it looks (facing from north toward east, pitch up from level).
- **Written by:** hearing the window, going to a mark and flying. **Read by:** laying out the panel.
- **Files:** src/camera.rs.

### the clocks

- **A store** on the processor.
- **What it is:** The frame's recent lengths, each pass's recent times on the processor, and the card's on the card.
- **Written by:** measuring. **Read by:** laying out the panel.
- **Limit:** recent frames kept, a pass, 240.
- **Limit:** frames on their way back from the card at once, 4.
- **Files:** src/clocks.rs.

### the wiring

- **A store** on the processor.
- **What it is:** This list: every lane, pass and store, fixed in the game. The frame runs in its order; the software map is drawn from it.
- **Written by:** none. **Read by:** laying out the panel.
- **Files:** src/wiring.rs.

### the panel's shapes

- **A store** on the processor.
- **What it is:** The panels cut into shapes for the card (made afresh each frame).
- **Written by:** laying out the panel. **Read by:** the panel.
- **Files:** src/panel.rs.

### the screen's picture

- **A store** on the card.
- **What it is:** The picture the card draws on and the screen shows, the window's size.
- **Written by:** the panel. **Read by:** showing it.
- **Files:** src/card.rs.

### the flight's report

- **A store** on the disk.
- **What it is:** A measured flight's report, logs/flight-<seconds since 1970>.txt (only on a flight).
- **Written by:** measuring. **Read by:** none.
- **Files:** src/flight.rs.

### the session's log

- **A store** on the disk.
- **What it is:** The session's messages and any crash, logs/session-<seconds since 1970>.log. Every part writes to it, so it has no lines on the map.
- **Written by:** none. **Read by:** none.
- **Files:** src/session.rs.

### the export

- **A store** on the disk.
- **What it is:** Old's test world written out as columns, exports/nice25 test: where it lies, the sea's level, every profile, and a file a tile (512 x 512 columns, raw deflate). The tablet takes its place at 3.1.
- **Written by:** none. **Read by:** the doorway.
- **Files:** src/doorway.rs.

### a tile's columns

- **A store** on the processor.
- **What it is:** A tile's answer: each column's ground, water and profile as the export holds them, its runs made when asked.
- **Written by:** the doorway. **Read by:** going to a mark.
- **Files:** src/doorway.rs.

### the marks

- **A store** on the disk.
- **What it is:** The places every flight stands at, the same each time, data/marks.toml: where, how high, which way, fog, whose marks (near or far), and what's changed there first. The game starts at the first.
- **Written by:** none. **Read by:** going to a mark.
- **Files:** src/marks.rs.

### what changed

- **A store** on the processor.
- **What it is:** The changes made through the doorway, by tile: whole blocks and bars, in the order they were made. A mark's are taken back when the flight moves on.
- **Written by:** going to a mark. **Read by:** the doorway.
- **Files:** src/doorway.rs.


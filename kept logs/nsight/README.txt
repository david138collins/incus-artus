Nsight Graphics GPU traces of Incus Artus, 2026-11-09 and 2026-11-10 (Builder)
=============================================================================

Copied from Builder's session scratchpad so nothing lives only in a temp folder (David: "Yes, copy them").
logs/ is ignored by git: this is a copy on disk, not a commit. The measured findings are written up in
docs/dyno.md (the dyno's ledger) and docs/review-2026-11-10.md (the whole-software review).

Machine: David's laptop, NVIDIA GeForce RTX 5070 Ti Laptop GPU (Blackwell GB20x), DirectX 12, driver 32.0.16.2101,
screen 2560 x 1600 at 240 Hz. Each trace is 3 frames, taken by Nsight's command line ("GPU Trace Profiler",
--real-time-shader-profiler, --set-gpu-clocks unaltered, --metric-set-id 0), the game started under it.
Unless said otherwise the game flew the dyno's route (INCUS_DYNO=baked, INCUS_DYNO_LAPS=1) in the test world
"nice25 test" with the presets render/dyno, camera/dyno, player/dyno; "100 s in" is over the land, "125 s in" out
at sea.

Registers: a trace file keeps each shader's registers only in a form Nsight's window reads (Shader Pipelines tab,
scrolled right: # Warp, # Reg, Live Registers, Instruction Mix). David read them out; they're copied below. The
other numbers (each shader's share of the samples, its top stalls, occupancy, throughput) come from Nsight's MCP
reader (scripts/, below).


The traces
----------

trace_sea/         2026-11-09 19:47, build ~101/102 (the march in one pass, march_pass), at sea 125 s.
                   The first look inside the march: the card kept 13 to 17% of the work in flight it could;
                   top states "Wait", "Selected", "No Instructions" (the program too big for the instruction
                   cache); about 44% of each group of 32 active. David's readout: march_pass 128 registers,
                   30,600 instructions. (docs/dyno.md, "The first look inside the march")

trace_twin/        2026-11-09 20:20, build ~102 with a twin variant of the march flown every other frame: two
                   march_pass rows (the game's and the variant's), taken to see what the compiler did to the
                   slower variant.

trace104_land/     2026-11-10 07:43, build 104 (the march in stages with light per face), over the land 100 s.
                   Samples: march_hit 32%, shade_staged 29%, face_light_staged 19%, sight_grounds_pass 15%,
                   listed_staged 3%, upscale 2%. Compute warps active about 10% of peak; top stall "Wait" in
                   every big pass; "No Instructions" down to 0.5%. queries/ holds its metrics, dispatches,
                   markers, frames and hotspots as read.

trace104_sea/      2026-11-10 07:45, build 104, at sea 125 s. Samples: march_hit 62%, shade_staged 17%,
                   face_light_staged 9%, grounds 4%, listed 3%, upscale 3%.
                   David's register readout (this trace):
                     march_hit          125 registers (122 live), 16 of 48 groups, 12.2K instructions
                     shade_staged        72 (70), 28 groups, 9.14K, shared memory 1536
                     face_light_staged   56 (54), 36 groups, 7.54K
                     sight_grounds_pass 255 (252), 8 groups, 24.1K (spills)
                     listed_staged       33 (29), 48 groups, 624
                     brought_up (upscale, pixel) 20 (18), 440
                     sight_cells_pass   168 (166), 12 groups, 4.94K, shared memory 5120
                     sight_choices_pass 168 (160), 12 groups, 10.5K, shared memory 8192
                     start_pass          48 (44), 40 groups, 3.25K

trace_p8/          2026-11-10 08:27, build 104 with probes/march_p8_far_only.wgsl as the twin (the near walk
                   switched off: the far march alone), at sea 125 s. Two march_hit rows. David's readout: the
                   probe's march_hit 121 registers (117 live), 16 groups, 9.27K instructions; the game's 125.
                   (The probe file in probes/ was remade later from build 107's march; this trace flew the
                   build 104 version.)

trace_p9/          2026-11-10 08:29, build 104 with probes/march_p9_near_only.wgsl as the twin (the near walk
                   alone, nothing past the blocks), at sea 125 s. David's readout: the probe's march_hit 80
                   registers (78 live), 24 groups (half the card), 10.8K instructions. (Same note on the probe
                   file as trace_p8.)

trace106_stages/   2026-11-10 09:59, build 106 with the near ground's grounds pass cut in two (work in progress,
                   later dropped), over the land 100 s. David's readout: sight_uncarved_pass 255 registers, 8
                   groups, 19.7K instructions (still spilling); the carving stage not sampled (4.48K).

trace106_stages4/  2026-11-10 10:16, build 106 with the grounds in four stages (later dropped), over the land.
                   David's readout: sight_parts_pass 255 (12.6K), sight_uncarved_pass 53 (5.42K, 36 groups),
                   sight_grounds_pass (the carve) 96 (4.48K, 20 groups); the maps stage not sampled (2.26K).
                   The stages lost to the single pass in flight (docs/dyno.md); the patch is on file in
                   Builder's notes.

trace_fill/        2026-11-10 12:55, build 107, NOT the dyno: the test world loaded and the player standing at
                   the spawn 40 s in, the live levels filling. Frames 6.0 to 7.5 ms; march_hit 41%,
                   shade_staged 35%, face_light_staged 13.5%, the tile passes under 3% (cells 1.4%, choices
                   0.7%). Warps active 18.6%.

trace_spawn_p9/    2026-11-10 13:27, build 107, the soak standing still at the spawn (INCUS_SOAK=90,
                   INCUS_SOAK_SPEED=0, INCUS_SOAK_ABOVE=2) 60 s in, with probes/march_p9_near_only.wgsl (remade
                   from build 107) as the twin. The game's march_hit 46.0% of samples (2 frames, 9.8 and 8.4 ms)
                   against the near-walk-only twin's 3.5% (1 frame, 2.9 ms): the near walk about 15% of the
                   march a frame at the spawn.

trace_spawn_p8/    2026-11-10 13:28, as trace_spawn_p9 with probes/march_p8_far_only.wgsl as the twin. Not
                   usable for the split: without the near walk the far march wades through the near ground and
                   costs more than the whole march.


scripts/
--------

mcp.py              a small client for Nsight's MCP reader (ngfx-mcp.exe, started from Nsight's folder with
                    --transport http --port 7661; it must be stopped before Nsight's window opens a trace).
q.py, q2.py         a query on an opened trace (q2 saves the whole result).
read_trace.py       a trace's frames, each shader's share and stalls, occupancy, dispatches.
read_twin_trace.py  each shader's share by hash (a twin's passes beside the game's).
hot104.py           each big pass's hotspots by function (they come back as intermediate lines with no names).
open104.py          opens build 104's two traces.
trace.sh, trace_mcp.py   earlier ways of taking a trace (the .bat files through Explorer are what worked).
trace*.bat          each trace's command line, as run (started with explorer.exe so the trace takes the
                    desktop's environment, not the agent's shell's).

queries/            what the reader returned for the build 104 traces (metrics, actions, markers, frames,
                    shader rows, hotspots).

probes/             the probe shaders flown as twins (each a march.wgsl with one part changed, never shipped),
                    and the scripts that make them from the current march.wgsl. A twin is made from the march as
                    it is the day it flies (a stale one read 13% wrong on 2026-11-10).

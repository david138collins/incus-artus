@echo off
rem The view near the ground (standing still at the test world's spawn, two blocks up: the soak at speed 0) traced with
rem a probe as the twin, so the trace holds its march_hit beside the game's: P9 the near walk alone, then P8 the far
rem march alone. 60 s in. Started through Explorer.
set S=C:\Users\David\AppData\Local\Temp\claude\ab96f937839e9dd4\70143432-62ed-4d7c-ac57-8161597e5036\scratchpad
cd /d "C:\Program Files\NVIDIA Corporation\Nsight Graphics 2026.4.0\host\windows-desktop-nomad-x64"
mkdir "%S%\trace_spawn_p7c" 2>nul
ngfx.exe --activity "GPU Trace Profiler" --exe "C:\Users\David\Desktop\Incus Artus\target\release\incus_artus.exe" --dir "C:\Users\David\Desktop\Incus Artus" --env "CARGO_MANIFEST_DIR=C:\Users\David\Desktop\Incus Artus; INCUS_SOAK=90; INCUS_SOAK_SPEED=0; INCUS_SOAK_ABOVE=2; INCUS_LOAD_WORLD=nice25 test; INCUS_START_PRESET=render/dyno,camera/dyno,player/dyno; INCUS_SIGHT_TWIN=%S%\march_p7c_cones_one.wgsl;" --start-after-ms 60000 --max-duration-ms 1000 --limit-to-frames 3 --architecture "Blackwell GB20x" --metric-set-id 0 --real-time-shader-profiler --set-gpu-clocks unaltered --auto-export --output-dir "%S%\trace_spawn_p7c" > "%S%\trace_spawn_p7c\bat-out.txt" 2>&1
echo done > "%S%\trace_spawn_p7c_done.txt"

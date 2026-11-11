@echo off
rem Build 104 traced by Nsight Graphics with a probe flown as the twin (every other timed frame), so the trace holds
rem the twin's march_hit beside the game's: P8 the far march alone, then P9 the near walk alone. Out at sea, 125 s in.
set S=C:\Users\David\AppData\Local\Temp\claude\ab96f937839e9dd4\70143432-62ed-4d7c-ac57-8161597e5036\scratchpad
cd /d "C:\Program Files\NVIDIA Corporation\Nsight Graphics 2026.4.0\host\windows-desktop-nomad-x64"
mkdir "%S%\trace_p8" 2>nul
mkdir "%S%\trace_p9" 2>nul
ngfx.exe --activity "GPU Trace Profiler" --exe "C:\Users\David\Desktop\Incus Artus\target\release\incus_artus.exe" --dir "C:\Users\David\Desktop\Incus Artus" --env "CARGO_MANIFEST_DIR=C:\Users\David\Desktop\Incus Artus; INCUS_DYNO=baked; INCUS_DYNO_LAPS=1; INCUS_LOAD_WORLD=nice25 test; INCUS_START_PRESET=render/dyno,camera/dyno,player/dyno; INCUS_SIGHT_TWIN=%S%\march_p8_far_only.wgsl;" --start-after-ms 125000 --max-duration-ms 1000 --limit-to-frames 3 --architecture "Blackwell GB20x" --metric-set-id 0 --real-time-shader-profiler --set-gpu-clocks unaltered --auto-export --output-dir "%S%\trace_p8" > "%S%\trace_p8\bat-out.txt" 2>&1
echo %ERRORLEVEL% > "%S%\trace_p8\bat-exit.txt"
ngfx.exe --activity "GPU Trace Profiler" --exe "C:\Users\David\Desktop\Incus Artus\target\release\incus_artus.exe" --dir "C:\Users\David\Desktop\Incus Artus" --env "CARGO_MANIFEST_DIR=C:\Users\David\Desktop\Incus Artus; INCUS_DYNO=baked; INCUS_DYNO_LAPS=1; INCUS_LOAD_WORLD=nice25 test; INCUS_START_PRESET=render/dyno,camera/dyno,player/dyno; INCUS_SIGHT_TWIN=%S%\march_p9_near_only.wgsl;" --start-after-ms 125000 --max-duration-ms 1000 --limit-to-frames 3 --architecture "Blackwell GB20x" --metric-set-id 0 --real-time-shader-profiler --set-gpu-clocks unaltered --auto-export --output-dir "%S%\trace_p9" > "%S%\trace_p9\bat-out.txt" 2>&1
echo %ERRORLEVEL% > "%S%\trace_p9\bat-exit.txt"
echo done > "%S%\trace_p89_done.txt"

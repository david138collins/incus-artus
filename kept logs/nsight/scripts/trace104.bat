@echo off
rem Build 104 (the stages with light per face) traced by Nsight Graphics' command line, started through Explorer
rem (its environment, not the agent's shell's): a GPU trace over the land (100 s in), then one out at sea (125 s in),
rem each its own flight, exported.
set S=C:\Users\David\AppData\Local\Temp\claude\ab96f937839e9dd4\70143432-62ed-4d7c-ac57-8161597e5036\scratchpad
cd /d "C:\Program Files\NVIDIA Corporation\Nsight Graphics 2026.4.0\host\windows-desktop-nomad-x64"
mkdir "%S%\trace104_land" 2>nul
mkdir "%S%\trace104_sea" 2>nul
ngfx.exe --activity "GPU Trace Profiler" --exe "C:\Users\David\Desktop\Incus Artus\target\release\incus_artus.exe" --dir "C:\Users\David\Desktop\Incus Artus" --env "CARGO_MANIFEST_DIR=C:\Users\David\Desktop\Incus Artus; INCUS_DYNO=baked; INCUS_DYNO_LAPS=1; INCUS_LOAD_WORLD=nice25 test; INCUS_START_PRESET=render/dyno,camera/dyno,player/dyno;" --start-after-ms 100000 --max-duration-ms 1000 --limit-to-frames 3 --architecture "Blackwell GB20x" --metric-set-id 0 --real-time-shader-profiler --set-gpu-clocks unaltered --auto-export --output-dir "%S%\trace104_land" > "%S%\trace104_land\bat-out.txt" 2>&1
echo %ERRORLEVEL% > "%S%\trace104_land\bat-exit.txt"
ngfx.exe --activity "GPU Trace Profiler" --exe "C:\Users\David\Desktop\Incus Artus\target\release\incus_artus.exe" --dir "C:\Users\David\Desktop\Incus Artus" --env "CARGO_MANIFEST_DIR=C:\Users\David\Desktop\Incus Artus; INCUS_DYNO=baked; INCUS_DYNO_LAPS=1; INCUS_LOAD_WORLD=nice25 test; INCUS_START_PRESET=render/dyno,camera/dyno,player/dyno;" --start-after-ms 125000 --max-duration-ms 1000 --limit-to-frames 3 --architecture "Blackwell GB20x" --metric-set-id 0 --real-time-shader-profiler --set-gpu-clocks unaltered --auto-export --output-dir "%S%\trace104_sea" > "%S%\trace104_sea\bat-out.txt" 2>&1
echo %ERRORLEVEL% > "%S%\trace104_sea\bat-exit.txt"
echo done > "%S%\trace104_done.txt"

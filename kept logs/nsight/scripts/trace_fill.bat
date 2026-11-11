@echo off
rem The first impression: the test world loaded and the player standing at the spawn while the live levels fill (no
rem dyno flight), traced by Nsight Graphics' command line 40 s in, started through Explorer.
set S=C:\Users\David\AppData\Local\Temp\claude\ab96f937839e9dd4\70143432-62ed-4d7c-ac57-8161597e5036\scratchpad
cd /d "C:\Program Files\NVIDIA Corporation\Nsight Graphics 2026.4.0\host\windows-desktop-nomad-x64"
mkdir "%S%\trace_fill" 2>nul
ngfx.exe --activity "GPU Trace Profiler" --exe "C:\Users\David\Desktop\Incus Artus\target\release\incus_artus.exe" --dir "C:\Users\David\Desktop\Incus Artus" --env "CARGO_MANIFEST_DIR=C:\Users\David\Desktop\Incus Artus; INCUS_LOAD_WORLD=nice25 test; INCUS_START_PRESET=render/dyno,camera/dyno,player/dyno;" --start-after-ms 40000 --max-duration-ms 1000 --limit-to-frames 3 --architecture "Blackwell GB20x" --metric-set-id 0 --real-time-shader-profiler --set-gpu-clocks unaltered --auto-export --output-dir "%S%\trace_fill" > "%S%\trace_fill\bat-out.txt" 2>&1
echo %ERRORLEVEL% > "%S%\trace_fill\bat-exit.txt"
echo done > "%S%\trace_fill_done.txt"

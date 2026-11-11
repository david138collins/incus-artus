@echo off
rem The near ground's grounds in two stages, traced by Nsight Graphics' command line over the land (100 s in), started
rem through Explorer: for David's register read of sight_uncarved_pass and sight_grounds_pass.
set S=C:\Users\David\AppData\Local\Temp\claude\ab96f937839e9dd4\70143432-62ed-4d7c-ac57-8161597e5036\scratchpad
cd /d "C:\Program Files\NVIDIA Corporation\Nsight Graphics 2026.4.0\host\windows-desktop-nomad-x64"
mkdir "%S%\trace106_stages4" 2>nul
ngfx.exe --activity "GPU Trace Profiler" --exe "C:\Users\David\Desktop\Incus Artus\target\release\incus_artus.exe" --dir "C:\Users\David\Desktop\Incus Artus" --env "CARGO_MANIFEST_DIR=C:\Users\David\Desktop\Incus Artus; INCUS_DYNO=baked; INCUS_DYNO_LAPS=1; INCUS_LOAD_WORLD=nice25 test; INCUS_START_PRESET=render/dyno,camera/dyno,player/dyno;" --start-after-ms 100000 --max-duration-ms 1000 --limit-to-frames 3 --architecture "Blackwell GB20x" --metric-set-id 0 --real-time-shader-profiler --set-gpu-clocks unaltered --auto-export --output-dir "%S%\trace106_stages4" > "%S%\trace106_stages4\bat-out.txt" 2>&1
echo %ERRORLEVEL% > "%S%\trace106_stages4\bat-exit.txt"
echo done > "%S%\trace106b_done.txt"

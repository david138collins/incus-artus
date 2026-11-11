@echo off
rem The dyno's flight traced by Nsight Graphics' command line, started through Explorer (its environment, not the
rem agent's shell's): a GPU trace out at sea, 125 s in, exported.
set OUT=C:\Users\David\AppData\Local\Temp\claude\ab96f937839e9dd4\70143432-62ed-4d7c-ac57-8161597e5036\scratchpad\trace_sea
cd /d "C:\Program Files\NVIDIA Corporation\Nsight Graphics 2026.4.0\host\windows-desktop-nomad-x64"
ngfx.exe --activity "GPU Trace Profiler" --exe "C:\Users\David\Desktop\Incus Artus\target\release\incus_artus.exe" --dir "C:\Users\David\Desktop\Incus Artus" --env "CARGO_MANIFEST_DIR=C:\Users\David\Desktop\Incus Artus; INCUS_DYNO=baked; INCUS_DYNO_LAPS=1; INCUS_LOAD_WORLD=nice25 test; INCUS_START_PRESET=render/dyno,camera/dyno,player/dyno;" --start-after-ms 125000 --max-duration-ms 1000 --limit-to-frames 3 --architecture "Blackwell GB20x" --metric-set-id 0 --real-time-shader-profiler --set-gpu-clocks unaltered --auto-export --output-dir "%OUT%" > "%OUT%\bat-out.txt" 2>&1
echo %ERRORLEVEL% > "%OUT%\bat-exit.txt"

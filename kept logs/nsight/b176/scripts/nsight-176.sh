#!/usr/bin/env bash
# Build 176 under Nsight's GPU Trace Profiler, standing at a mark: 20 frames from 100 s in, the real-time shader
# profiler on, the card's clocks left alone. Three runs: the mountain top and high over the spawn (the marks whose
# time isn't their longest ray's), and the sea (the trace that stood at the spawn last time). No code changes while
# it runs. The env string ends at the last value's ";" with nothing after it: a stray "; " became part of a mark's
# name on 2026-11-14.
O="C:/Users/David/AppData/Local/Temp/claude/ab96f937839e9dd4/70bc1fc4-3a14-4888-8de0-a073bfe21d8d/scratchpad/nsight"
G='C:\Users\David\Desktop\Incus Artus'
P="/c/Users/David/Desktop/Incus Artus"
cd "/c/Program Files/NVIDIA Corporation/Nsight Graphics 2026.4.0/host/windows-desktop-nomad-x64" || exit 1
trace() {
  local name=$1 mark=$2
  mkdir -p "$O/$name"
  echo "=== trace $name: at $mark ($(date +%H:%M:%S))"
  timeout 300 ./ngfx.exe --activity "GPU Trace Profiler" --exe "$G\\target\\release\\incus_artus.exe" --dir "$G" \
    --env "CARGO_MANIFEST_DIR=$G; INCUS_MARK=$mark;" --start-after-ms 100000 --limit-to-frames 20 --auto-export \
    --set-gpu-clocks unaltered --real-time-shader-profiler --output-dir "$(cygpath -w "$O/$name")" > "$O/$name/ngfx-out.txt" 2>&1
  echo "    ended $? ($(date +%H:%M:%S)); $(ls "$O/$name" | grep -c gputrace) trace file(s)"
  local log; log="$P/logs/$(ls -t "$P/logs" | grep '^session-' | head -1)"
  echo "    stood: $(grep -m1 "incus_artus::runner: At " "$log" | sed 's/.*runner: //' | cut -c1-70); $(grep -c "is no mark's name" "$log") warnings about the mark's name; $(grep -m1 -o "build [0-9]* ([0-9a-f]*)" "$log")"
  taskkill //IM incus_artus.exe //F >/dev/null 2>&1
  sleep 4
}
trace "b176-mountain" "the mountain top on a clear day"
trace "b176-high" "high over the spawn"
trace "b176-sea" "low over the sea"
echo "=== all three traced ($(date +%H:%M:%S))"

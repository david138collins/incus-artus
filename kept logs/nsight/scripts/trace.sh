#!/usr/bin/env bash
# A GPU trace (Nsight Graphics, its command line) of the dyno's flight on nice25 test: the game started under the
# tool, the trace taken a while in (by default 100 s: the measured lap's first leg, over the land), then exported.
# Usage: trace.sh NAME [START_MS] [extra ngfx options...]
P="C:/Users/David/Desktop/Incus Artus"
S="C:/Users/David/AppData/Local/Temp/claude/ab96f937839e9dd4/70143432-62ed-4d7c-ac57-8161597e5036/scratchpad"
NGFX="/c/Program Files/NVIDIA Corporation/Nsight Graphics 2026.4.0/host/windows-desktop-nomad-x64/ngfx.exe"
name=$1; shift
start=${1:-100000}; shift
out="$S/trace_$name"
mkdir -p "$out"
cd "$(dirname "$NGFX")" && "$NGFX" --activity "GPU Trace Profiler" \
  --exe "$P/target/release/incus_artus.exe" --dir "$P" \
  --env "CARGO_MANIFEST_DIR=$P; INCUS_DYNO=baked; INCUS_DYNO_LAPS=1; INCUS_LOAD_WORLD=nice25 test; INCUS_START_PRESET=render/dyno,camera/dyno,player/dyno;" \
  --start-after-ms "$start" --max-duration-ms 1000 --limit-to-frames 3 \
  --architecture "Blackwell GB20x" --metric-set-id 0 --real-time-shader-profiler --set-gpu-clocks unaltered \
  --auto-export --output-dir "$out" "$@" > "$out/ngfx.log" 2>&1
echo "exit $?"
ls -la "$out" | head -40

#!/usr/bin/env bash
# Build 175 (build 174's march; the shader text on a switch) under Nsight's GPU Trace Profiler, standing at a mark: 20 frames from 100 s in (the far eye whole by about
# 60 s), the real-time shader profiler on, the card's clocks left alone. Four runs: the plain with the height table and
# without its true numbers, the spawn, the sea. The game as it's built (target/release, build 174): no code changes
# while it runs.
O="C:/Users/David/AppData/Local/Temp/claude/ab96f937839e9dd4/70bc1fc4-3a14-4888-8de0-a073bfe21d8d/scratchpad/nsight"
G='C:\Users\David\Desktop\Incus Artus'
cd "/c/Program Files/NVIDIA Corporation/Nsight Graphics 2026.4.0/host/windows-desktop-nomad-x64" || exit 1
trace() {
  local name=$1 mark=$2 extra=$3
  mkdir -p "$O/$name"
  echo "=== trace $name: at $mark $extra ($(date +%H:%M:%S))"
  timeout 300 ./ngfx.exe --activity "GPU Trace Profiler" --exe "$G\\target\\release\\incus_artus.exe" --dir "$G" \
    --env "CARGO_MANIFEST_DIR=$G; INCUS_MARK=$mark; $extra" --start-after-ms 100000 --limit-to-frames 20 --auto-export \
    --set-gpu-clocks unaltered --real-time-shader-profiler --output-dir "$(cygpath -w "$O/$name")" > "$O/$name/ngfx-out.txt" 2>&1
  echo "    ended $? ($(date +%H:%M:%S)); $(ls "$O/$name" | grep -c gputrace) trace file(s)"
  taskkill //IM incus_artus.exe //F >/dev/null 2>&1
  sleep 4
}
trace "b175-plain" "the bare plain" "INCUS_SHADER_TEXT=1;"
trace "b175-plain-notable" "the bare plain" "INCUS_FAR_TOPS=0;"
trace "b175-spawn" "on the ground at the spawn" ""
trace "b175-sea" "low over the sea" ""
echo "=== all four traced ($(date +%H:%M:%S))"

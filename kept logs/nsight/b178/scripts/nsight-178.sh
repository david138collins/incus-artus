#!/usr/bin/env bash
# Build 178 under Nsight's GPU Trace Profiler, standing at the mountain top: 20 frames from 100 s in, for the row of
# the march once the textures are in (march_all: its registers, the groups that fit, Smem). No code changes while
# it runs.
# Run from anywhere: bash "kept logs/nsight/b178/scripts/nsight-178.sh". The traces land in logs/nsight/b178/ (on the
# laptop only). The env string ends at the last value's ";" with nothing after it: a stray "; " became part of a mark's
# name on 2026-11-14.
O="C:/Users/David/Desktop/Incus Artus/logs/nsight/b178"
G='C:\Users\David\Desktop\Incus Artus'
P="/c/Users/David/Desktop/Incus Artus"
cd "/c/Program Files/NVIDIA Corporation/Nsight Graphics 2026.4.0/host/windows-desktop-nomad-x64" || exit 1
trace() {
  local name=$1 mark=$2
  mkdir -p "$O/$name"
  echo "=== trace $name: at $mark ($(date +%H:%M:%S))"
  timeout 420 ./ngfx.exe --activity "GPU Trace Profiler" --exe "$G\\target\\release\\incus_artus.exe" --dir "$G" \
    --env "CARGO_MANIFEST_DIR=$G; INCUS_MARK=$mark; INCUS_SHADER_TEXT=1;" --start-after-ms 100000 --limit-to-frames 20 --auto-export \
    --set-gpu-clocks unaltered --real-time-shader-profiler --output-dir "$(cygpath -w "$O/$name")" > "$O/$name/ngfx-out.txt" 2>&1
  echo "    ended $? ($(date +%H:%M:%S)); $(ls "$O/$name" | grep -c gputrace) trace file(s)"
  local log; log="$P/logs/$(ls -t "$P/logs" | grep '^session-' | head -1)"
  echo "    stood: $(grep -m1 "incus_artus::runner: At " "$log" | sed 's/.*runner: //' | cut -c1-70); $(grep -c "is no mark's name" "$log") warnings about the mark's name; $(grep -m1 -o "build [0-9]* ([0-9a-f]*)" "$log"); $(grep -c "by the march once the textures are in" "$log") line(s) saying the second program took over"
  taskkill //IM incus_artus.exe //F >/dev/null 2>&1
  sleep 4
}
trace "b178-mountain" "the mountain top on a clear day"
echo "=== traced ($(date +%H:%M:%S))"

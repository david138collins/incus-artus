"""A first look at the saved tables of each trace: python look.py TRACE_FOLDER... [--metrics SUBSTR,SUBSTR]"""
import json
import os
import sys
from collections import defaultdict

folders = [a for a in sys.argv[1:] if not a.startswith("--")]
want = []
for a in sys.argv[1:]:
    if a.startswith("--metrics="):
        want = a.split("=", 1)[1].split(",")


def load(folder, entity):
    data = json.load(open(os.path.join(folder, entity + ".json"), encoding="utf-8"))
    cols = data["columns"]
    return [dict(zip(cols, r)) for r in data["rows"]]


for folder in folders:
    print("=====", os.path.basename(folder))
    frames = load(folder, "frames")
    if frames:
        keys = [k for k in frames[0] if "dur" in k.lower() or "ms" in k.lower()]
        print("frames:", len(frames), "keys", list(frames[0].keys())[:8], [round(f.get(keys[0], 0), 2) for f in frames[:6]] if keys else "")
    for s in load(folder, "shader_profiler"):
        print("  shader %-28s %-8s %6.2f%%  samples %-8s lanes/warp %-6s occupancy %-6s divergence %-6s stalls %s hash %s" % (
            s.get("shader"), s.get("shaderStage"), s.get("samplePercent") or 0, s.get("samples"), s.get("activeLanesPerWarp"), s.get("occupancy"), s.get("warpDivergencePercent"), s.get("dominantStalls"), s.get("shaderHash")))
    markers = load(folder, "markers")
    labels = defaultdict(int)
    for m in markers:
        labels[m.get("label")] += 1
    print("  markers:", dict(labels))
    actions = load(folder, "actions")
    kinds = defaultdict(lambda: [0, 0.0])
    for a in actions:
        kinds[a.get("action.kind")][0] += 1
        kinds[a.get("action.kind")][1] += a.get("duration_us") or 0.0
    print("  actions:", {k: (n, round(us / 1000.0, 2)) for k, (n, us) in kinds.items()}, "(count, ms over the trace)")
    metrics = load(folder, "metrics")
    if metrics:
        print("  metric keys:", list(metrics[0].keys()))
    for m in metrics:
        name = str(m.get("name"))
        if want and any(w in name for w in want):
            print("    %-110s %s %s" % (name, m.get("value"), {k: v for k, v in m.items() if k not in ("name", "value", "id") and v is not None}))

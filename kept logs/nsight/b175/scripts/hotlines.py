"""A shader's hot lines from a trace's saved hotspots, read against a DXIL listing:
python hotlines.py TRACE_FOLDER SHADER LISTING [TOP] [OFFSET]"""
import io
import json
import os
import re
import sys

folder, shader, listing = sys.argv[1], sys.argv[2], sys.argv[3]
top = int(sys.argv[4]) if len(sys.argv) > 4 else 40
offset = int(sys.argv[5]) if len(sys.argv) > 5 else 0
lines = io.open(listing, encoding="utf-8", errors="replace").read().replace("\r\n", "\n").split("\n")
data = json.load(open(os.path.join(folder, "shader_profiler_hotspots.json"), encoding="utf-8"))
cols = data["columns"]
rows = [dict(zip(cols, r)) for r in data["rows"] if dict(zip(cols, r)).get("shader") == shader]
kinds = sorted(set(r.get("sourceType") for r in rows))
print("source types:", kinds, "rows", len(rows))
rows = [r for r in rows if r.get("sourceType") == "intermediate"]
total = sum(r["samples"] for r in rows)
rows.sort(key=lambda r: -r["samples"])
print("%d intermediate rows, %d samples; the listing has %d lines" % (len(rows), total, len(lines)))
for r in rows[:top]:
    n = r.get("sourceLine")
    text = lines[n - 1 + offset].strip() if n and 0 < n + offset <= len(lines) else "?"
    print("%5.1f%%  line %-5s %-44s %s" % (100.0 * r["samples"] / max(total, 1), n, ",".join(r.get("dominantStalls") or [])[:44], text[:150]))

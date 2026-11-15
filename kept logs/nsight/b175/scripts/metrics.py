"""Chosen metrics of several traces side by side: python metrics.py SUBSTR,SUBSTR TRACE_FOLDER..."""
import json
import os
import sys

want = sys.argv[1].split(",")
folders = sys.argv[2:]
tables = []
for folder in folders:
    data = json.load(open(os.path.join(folder, "metrics.json"), encoding="utf-8"))
    cols = data["columns"]
    rows = [dict(zip(cols, r)) for r in data["rows"]]
    table = {}
    for m in rows:
        key = str(m.get("name"))
        if m.get("shader"):
            key += " [" + str(m["shader"]) + "]"
        if m.get("range") and m.get("range") != "full_trace":
            key += " {" + str(m["range"]) + "}"
        table.setdefault(key, m.get("value"))
    tables.append(table)
names = []
for t in tables:
    for k in t:
        if any(w in k for w in want) and k not in names:
            names.append(k)
print("%-112s" % "metric" + "".join("%16s" % os.path.basename(f)[-14:] for f in folders))
for k in sorted(names):
    vals = [t.get(k) for t in tables]
    if all(v in (None, 0, 0.0) for v in vals):
        continue
    print("%-112s" % k[:112] + "".join("%16s" % ("-" if v is None else ("%.4g" % v if isinstance(v, (int, float)) else str(v)[:14])) for v in vals))

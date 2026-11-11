"""Build 104's traces: each big pass's hotspots summed by function and by source type (python hot104.py)."""
import json
import os
import sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mcp

S = os.path.dirname(os.path.abspath(__file__))
arts = {"land": "3e166986-4c3f-45c1-bab1-629664000393/inspected-artifact/1", "sea": "3e166986-4c3f-45c1-bab1-629664000393/inspected-artifact/2"}
shaders = ["march_hit", "shade_staged", "face_light_staged", "sight_grounds_pass"]


def query(artifact, args):
    reply = mcp.request("tools/call", {"name": "ngfx_query", "arguments": dict(args, artifactId=artifact)})
    result = (reply or {}).get("result", {})
    if (reply or {}).get("error") or result.get("isError"):
        print("ERROR", json.dumps(reply)[:1500])
        return None
    data = result.get("structuredContent")
    if data is None:
        text = "\n".join(item.get("text", "") for item in result.get("content", []) if item.get("type") == "text")
        data = json.loads(text)
    return data


for place, artifact in arts.items():
    for shader in shaders:
        data = query(artifact, {"from": "shader_profiler_hotspots", "where": {"eq": ["shader", shader]}, "select": ["sourceType", "samples", "samplePercent", "function", "sourceLine", "dominantStalls", "activeLanesPerWarp", "instructionAddress"], "orderBy": [{"field": "samples", "direction": "desc"}], "limit": 1000})
        if not data:
            continue
        cols = data["columns"]
        i = {c: k for k, c in enumerate(cols)}
        rows = data["rows"]
        json.dump(data, open(os.path.join(S, f"hot104_{place}_{shader}.json"), "w", encoding="utf-8"))
        by_type = defaultdict(list)
        for r in rows:
            by_type[r[i["sourceType"]]].append(r)
        print(f"== {place} {shader}: {len(rows)} hotspot rows; types {dict((k, len(v)) for k, v in by_type.items())}")
        for kind, rs in by_type.items():
            total = sum(r[i["samples"]] for r in rs)
            fn = defaultdict(int)
            for r in rs:
                fn[r[i["function"]]] += r[i["samples"]]
            top = sorted(fn.items(), key=lambda kv: -kv[1])[:12]
            print(f"  {kind}: samples {total}; by function: " + ", ".join(f"{f}: {100.0 * n / max(total, 1):.1f}%" for f, n in top))
            for r in rs[:8]:
                print(f"    line {r[i['sourceLine']]} {r[i['function']]} {r[i['samplePercent']]:.2f}% {r[i['dominantStalls']]} lanes {r[i['activeLanesPerWarp']]}")

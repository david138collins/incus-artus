"""Open a GPU trace in Nsight's reader and print its frames, each shader's share and top stalls, and the occupancy:
python read_trace.py PATH"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mcp

path = sys.argv[1]


def call(tool, args):
    reply = mcp.request("tools/call", {"name": tool, "arguments": args})
    result = (reply or {}).get("result", {})
    if (reply or {}).get("error") or result.get("isError"):
        raise SystemExit(f"{tool} failed: {json.dumps(reply)[:1500]}")
    data = result.get("structuredContent")
    if data is None:
        text = "\n".join(item.get("text", "") for item in result.get("content", []) if item.get("type") == "text")
        data = json.loads(text)
    return data


mcp.request("tools/call", {"name": "ngfx_get_api_manifest", "arguments": {}})
artifact = call("ngfx_open_artifact", {"path": path})["artifact"]["artifactId"]
frames = call("ngfx_query", {"artifactId": artifact, "from": "frames", "limit": 10})
print("frames (ms):", [round(r[2], 2) for r in frames["rows"]])
shaders = call("ngfx_query", {"artifactId": artifact, "from": "shader_profiler", "select": ["shader", "samples", "samplePercent", "dominantStalls"], "orderBy": [{"field": "samples", "direction": "desc"}], "limit": 20})
for r in shaders["rows"]:
    print(f"  {r[0]:28} {r[2]:5.1f}%  {r[3]}")
metrics = call("ngfx_query", {"artifactId": artifact, "from": "metrics", "limit": 1000})
i = {c: k for k, c in enumerate(metrics["columns"])}
for r in metrics["rows"]:
    name = r[i["name"]]
    if any(k in name for k in ["warps_active_shader_compute.avg.pct", "sm__throughput.avg.pct", "warps_issue_stalled_no_instruction.avg.pct"]):
        print(f"  {name}: {r[i['value']]:.2f}")
actions = call("ngfx_query", {"artifactId": artifact, "from": "actions", "limit": 1000})
kinds = {}
for r in actions["rows"]:
    kinds.setdefault(r[3], [0, 0.0])
    kinds[r[3]][0] += 1
    kinds[r[3]][1] += r[4] or 0.0
print("actions:", {k: (n, round(us / 1000, 2)) for k, (n, us) in kinds.items()}, "(count, ms summed over the trace)")

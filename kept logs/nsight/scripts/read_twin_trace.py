"""Each shader's share in a trace, by hash (a twin's passes beside the game's): python read_twin_trace.py PATH [PATH...]"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mcp


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
for path in sys.argv[1:]:
    artifact = call("ngfx_open_artifact", {"path": path})["artifact"]["artifactId"]
    frames = call("ngfx_query", {"artifactId": artifact, "from": "frames", "limit": 10})
    print("==", os.path.basename(os.path.dirname(path)), "frames (ms):", [round(r[2], 2) for r in frames["rows"]])
    rows = call("ngfx_query", {"artifactId": artifact, "from": "shader_profiler", "select": ["shader", "samples", "samplePercent", "shaderHash"], "orderBy": [{"field": "samples", "direction": "desc"}], "limit": 20})
    for r in rows["rows"]:
        print(f"  {r[0]:24} {r[2]:5.1f}%  {r[1]:8d}  {r[3]}")

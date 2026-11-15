"""A trace's inventory, and one shader's hottest rows of each source type: python hot.py PATH SHADER [fields|rows]"""
import json
import os
import sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mcp


def call(tool, args):
    reply = mcp.request("tools/call", {"name": tool, "arguments": args})
    result = (reply or {}).get("result", {})
    if (reply or {}).get("error") or result.get("isError"):
        return {"ERROR": json.dumps(reply)[:1500]}
    data = result.get("structuredContent")
    if data is None:
        text = "\n".join(item.get("text", "") for item in result.get("content", []) if item.get("type") == "text")
        try:
            data = json.loads(text)
        except Exception:
            data = {"text": text}
    return data


path, shader = sys.argv[1], sys.argv[2]
what = sys.argv[3] if len(sys.argv) > 3 else "rows"
mcp.request("tools/call", {"name": "ngfx_get_api_manifest", "arguments": {}})
artifact = call("ngfx_open_artifact", {"path": path})["artifact"]["artifactId"]
if what == "fields":
    described = call("ngfx_describe_artifact", {"artifactId": artifact})
    print("inventory:", json.dumps(described)[:3000])
    for entity in ("shader_profiler", "shader_profiler_hotspots"):
        schema = call("ngfx_get_schema", {"artifactId": artifact, "entity": entity})
        for f in schema.get("schema", {}).get("fields", []):
            print("  %s.%s (%s): %s %s" % (entity, f["name"], f["valueType"], f["description"][:150], f.get("availabilityNotes") or ""))
    sys.exit(0)
fields = ["sourceType", "hotspotIndex", "samples", "samplePercent", "function", "sourcePath", "sourceLine", "dominantStalls", "activeLanesPerWarp", "instructionAddress"]
data = call("ngfx_query", {"artifactId": artifact, "from": "shader_profiler_hotspots", "where": {"eq": ["shader", shader]}, "select": fields, "orderBy": [{"field": "samples", "direction": "desc"}], "limit": 1000})
if "ERROR" in data:
    print(data)
    sys.exit(1)
i = {c: k for k, c in enumerate(data["columns"])}
by_type = defaultdict(list)
for r in data["rows"]:
    by_type[r[i["sourceType"]]].append(r)
json.dump(data, open(os.path.join(os.path.dirname(path), "hot_%s.json" % shader), "w", encoding="utf-8"))
for kind, rs in by_type.items():
    total = sum(r[i["samples"]] for r in rs)
    print("== %s: %d rows, %d samples" % (kind, len(rs), total))
    for r in rs[:40]:
        print("   line %-6s %-28s %6.2f%% of the trace  %5.1f%% of this shader  stalls %s lanes %s addr %s" % (r[i["sourceLine"]], str(r[i["function"]])[:28], r[i["samplePercent"]], 100.0 * r[i["samples"]] / max(total, 1), r[i["dominantStalls"]], r[i["activeLanesPerWarp"]], r[i["instructionAddress"]]))

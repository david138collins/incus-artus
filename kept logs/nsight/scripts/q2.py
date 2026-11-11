"""A query on an opened Nsight artifact, the whole result saved: python q2.py ARTIFACT ENTITY OUT.json [JSON extra args]."""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mcp

artifact, entity, out = sys.argv[1], sys.argv[2], sys.argv[3]
args = {"artifactId": artifact, "from": entity, "limit": 1000}
if len(sys.argv) > 4:
    args.update(json.loads(sys.argv[4]))
reply = mcp.request("tools/call", {"name": "ngfx_query", "arguments": args})
result = (reply or {}).get("result", {})
if (reply or {}).get("error") or result.get("isError"):
    print("ERROR:", json.dumps(reply)[:3000])
    sys.exit(1)
data = result.get("structuredContent")
if data is None:
    text = "\n".join(item.get("text", "") for item in result.get("content", []) if item.get("type") == "text")
    data = json.loads(text) if text.strip().startswith("{") else text
open(out, "w", encoding="utf-8").write(json.dumps(data, indent=1))
rows = data.get("rows") if isinstance(data, dict) else None
print("saved", out, "rows:", len(rows) if rows is not None else "?", "columns:", data.get("columns") if isinstance(data, dict) else "?")

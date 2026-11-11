"""Queries on an opened Nsight artifact: python q.py ARTIFACT schema ENTITY | query ENTITY [JSON extra args]."""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mcp

artifact, what, entity = sys.argv[1], sys.argv[2], sys.argv[3]
if what == "schema":
    mcp.show(mcp.request("tools/call", {"name": "ngfx_get_schema", "arguments": {"artifactId": artifact, "entity": entity}}))
else:
    args = {"artifactId": artifact, "from": entity, "limit": 50}
    if len(sys.argv) > 4:
        args.update(json.loads(sys.argv[4]))
    reply = mcp.request("tools/call", {"name": "ngfx_query", "arguments": args})
    result = (reply or {}).get("result", {})
    if (reply or {}).get("error") or result.get("isError"):
        print("ERROR:", json.dumps(reply)[:3000])
    else:
        data = result.get("structuredContent")
        if data is None:
            text = "\n".join(item.get("text", "") for item in result.get("content", []) if item.get("type") == "text")
            data = json.loads(text) if text.strip().startswith("{") else text
        print(json.dumps(data, indent=1)[:30000])

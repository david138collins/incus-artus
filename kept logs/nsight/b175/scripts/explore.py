"""Open a trace and print the reader's schemas and a first look at each table: python explore.py PATH"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mcp


def call(tool, args):
    reply = mcp.request("tools/call", {"name": tool, "arguments": args})
    result = (reply or {}).get("result", {})
    if (reply or {}).get("error") or result.get("isError"):
        return {"ERROR": json.dumps(reply)[:1200]}
    data = result.get("structuredContent")
    if data is None:
        text = "\n".join(item.get("text", "") for item in result.get("content", []) if item.get("type") == "text")
        try:
            data = json.loads(text)
        except Exception:
            data = {"text": text}
    return data


mcp.request("tools/call", {"name": "ngfx_get_api_manifest", "arguments": {}})
opened = call("ngfx_open_artifact", {"path": sys.argv[1]})
artifact = opened["artifact"]["artifactId"]
print("artifact", artifact)
for entity in sys.argv[2:]:
    schema = call("ngfx_get_schema", {"artifactId": artifact, "entity": entity})
    print("==", entity, "schema:", json.dumps(schema)[:2500])
    rows = call("ngfx_query", {"artifactId": artifact, "from": entity, "limit": 6})
    print("   first rows:", json.dumps(rows)[:2500])

"""Open each trace in Nsight's reader and save its tables beside it as JSON: python dump.py TRACE_FOLDER..."""
import glob
import json
import os
import sys

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


def table(artifact, entity):
    """Every row of a table with every field its schema names, a page at a time."""
    schema = call("ngfx_get_schema", {"artifactId": artifact, "entity": entity})
    fields = [f["name"] for f in schema.get("schema", {}).get("fields", [])]
    rows, offset, columns = [], 0, fields
    while True:
        page = call("ngfx_query", {"artifactId": artifact, "from": entity, "select": fields, "limit": 1000, "offset": offset})
        if "ERROR" in page:
            if offset == 0:
                # Without paging, or without the full select, as a fall-back.
                page = call("ngfx_query", {"artifactId": artifact, "from": entity, "select": fields, "limit": 1000})
                if "ERROR" in page:
                    return {"columns": [], "rows": [], "error": page["ERROR"]}
                return {"columns": page.get("columns", fields), "rows": page.get("rows", [])}
            break
        columns = page.get("columns", fields)
        got = page.get("rows", [])
        rows += got
        if len(got) < 1000:
            break
        offset += 1000
    return {"columns": columns, "rows": rows}


mcp.request("tools/call", {"name": "ngfx_get_api_manifest", "arguments": {}})
for folder in sys.argv[1:]:
    traces = glob.glob(os.path.join(folder, "*.ngfx-gputrace"))
    if not traces:
        print(folder, ": no trace")
        continue
    opened = call("ngfx_open_artifact", {"path": traces[0].replace("/", "\\")})
    if "ERROR" in opened:
        print(folder, opened)
        continue
    artifact = opened["artifact"]["artifactId"]
    counts = {}
    for entity in ("frames", "shader_profiler", "shader_profiler_hotspots", "metrics", "markers", "actions"):
        data = table(artifact, entity)
        json.dump(data, open(os.path.join(folder, entity + ".json"), "w", encoding="utf-8"))
        counts[entity] = len(data["rows"]) if not data.get("error") else data["error"][:200]
    print(os.path.basename(folder), counts)

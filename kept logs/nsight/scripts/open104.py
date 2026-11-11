"""Open build 104's two traces in Nsight's trace reader and print their artifact ids and what they hold."""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mcp

S = os.path.dirname(os.path.abspath(__file__))
traces = {
    "land": os.path.join(S, "trace104_land", "incus_artus_2026_11_10_07_43_20.ngfx-gputrace"),
    "sea": os.path.join(S, "trace104_sea", "incus_artus_2026_11_10_07_45_36.ngfx-gputrace"),
}
manifest = mcp.request("tools/call", {"name": "ngfx_get_api_manifest", "arguments": {}})
print("manifest ok" if manifest and "error" not in manifest else json.dumps(manifest)[:800])
ids = {}
for name, path in traces.items():
    reply = mcp.request("tools/call", {"name": "ngfx_open_artifact", "arguments": {"path": path}})
    result = (reply or {}).get("result", {})
    data = result.get("structuredContent")
    if data is None:
        text = "\n".join(item.get("text", "") for item in result.get("content", []) if item.get("type") == "text")
        data = json.loads(text) if text.strip().startswith("{") else text
    if (reply or {}).get("error") or result.get("isError"):
        print(name, "FAILED:", json.dumps(reply)[:2000])
        continue
    print("==", name, json.dumps(data)[:1500])
    artifact = data.get("artifactId") if isinstance(data, dict) else None
    if artifact:
        ids[name] = artifact
open(os.path.join(S, "trace104_ids.json"), "w").write(json.dumps(ids))
print(ids)

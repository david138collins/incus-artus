"""A tiny MCP client for Nsight Graphics' Workflow server (Streamable HTTP on 127.0.0.1:7661/mcp).

python mcp.py list                      -> tools/list (names and input schemas, short)
python mcp.py call NAME '{"arg": 1}'    -> tools/call, prints the result's text/JSON
The session id is kept in mcp_session.txt beside this script (initialized on first use).
"""
import json
import os
import sys
import urllib.request

URL = "http://127.0.0.1:7661/mcp"
HERE = os.path.dirname(os.path.abspath(__file__))
SESSION_FILE = os.path.join(HERE, "mcp_session.txt")
_next_id = [1]


def post(body, session=None, timeout=600):
    data = json.dumps(body).encode("utf-8")
    req = urllib.request.Request(URL, data=data, method="POST")
    req.add_header("Content-Type", "application/json")
    req.add_header("Accept", "application/json, text/event-stream")
    req.add_header("MCP-Protocol-Version", "2025-06-18")
    if session:
        req.add_header("Mcp-Session-Id", session)
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        sid = resp.headers.get("Mcp-Session-Id")
        kind = resp.headers.get("Content-Type", "")
        raw = resp.read().decode("utf-8", "replace")
    if "text/event-stream" in kind:
        messages = []
        for line in raw.splitlines():
            if line.startswith("data:"):
                payload = line[5:].strip()
                if payload:
                    try:
                        messages.append(json.loads(payload))
                    except json.JSONDecodeError:
                        pass
        # The response to our request (the one with an id), else the last.
        reply = next((m for m in messages if "id" in m and m.get("id") == body.get("id")), messages[-1] if messages else None)
        return sid, reply
    return sid, (json.loads(raw) if raw.strip() else None)


def session():
    if os.path.exists(SESSION_FILE):
        return open(SESSION_FILE).read().strip()
    sid, reply = post({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "builder", "version": "1"}}})
    if reply is not None:
        print("initialize:", json.dumps(reply.get("result", reply))[:600], file=sys.stderr)
    post({"jsonrpc": "2.0", "method": "notifications/initialized"}, sid)
    if sid:
        open(SESSION_FILE, "w").write(sid)
    return sid


def request(method, params):
    sid = session()
    rid = int.from_bytes(os.urandom(3), "big")
    _, reply = post({"jsonrpc": "2.0", "id": rid, "method": method, "params": params}, sid)
    return reply


def show(reply):
    if reply is None:
        print("(no reply)")
        return
    if "error" in reply:
        print("ERROR:", json.dumps(reply["error"], indent=1)[:4000])
        return
    result = reply.get("result", {})
    for item in result.get("content", []):
        if item.get("type") == "text":
            print(item["text"])
        else:
            print(json.dumps(item)[:2000])
    if "structuredContent" in result:
        print(json.dumps(result["structuredContent"], indent=1))
    if result.get("isError"):
        print("(isError)")


if __name__ == "__main__":
    what = sys.argv[1]
    if what == "list":
        reply = request("tools/list", {})
        for tool in reply.get("result", {}).get("tools", []):
            print("==", tool["name"], ":", tool.get("description", "")[:300].replace("\n", " "))
            print("   input:", json.dumps(tool.get("inputSchema", {}))[:1500])
    elif what == "call":
        args = json.loads(sys.argv[3]) if len(sys.argv) > 3 else {}
        show(request("tools/call", {"name": sys.argv[2], "arguments": args}))
    elif what == "reset":
        if os.path.exists(SESSION_FILE):
            os.remove(SESSION_FILE)

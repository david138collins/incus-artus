"""The game launched under Nsight Graphics' GPU Trace (its MCP server), the dyno flown, a trace taken when the
flight reaches a given moment (a log line, then a delay), the artifact described.

python trace_mcp.py NAME [DELAY_S] [LINE]   (defaults: 1 s after "Dyno: baked after": the warm-up lap's start,
over the land)
"""
import glob
import json
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mcp

PROJECT = "C:/Users/David/Desktop/Incus Artus"
HERE = os.path.dirname(os.path.abspath(__file__))
name = sys.argv[1]
delay = float(sys.argv[2]) if len(sys.argv) > 2 else 1.0
line = sys.argv[3] if len(sys.argv) > 3 else "Dyno: baked after"
out = os.path.join(HERE, "trace_" + name)
os.makedirs(out, exist_ok=True)


def call(tool, args, timeout=900):
    reply = mcp.request("tools/call", {"name": tool, "arguments": args})
    result = (reply or {}).get("result", {})
    text = "\n".join(item.get("text", "") for item in result.get("content", []) if item.get("type") == "text")
    structured = result.get("structuredContent")
    if (reply or {}).get("error") or result.get("isError"):
        print(f"{tool} FAILED:", json.dumps(reply)[:3000])
        sys.exit(1)
    return structured if structured is not None else (json.loads(text) if text.strip().startswith("{") else text)


logs_before = set(glob.glob(os.path.join(PROJECT, "logs", "session-*.log")))
env = {
    "CARGO_MANIFEST_DIR": PROJECT,
    "INCUS_DYNO": "baked",
    "INCUS_DYNO_LAPS": "1",
    "INCUS_LOAD_WORLD": "nice25 test",
    "INCUS_START_PRESET": "render/dyno,camera/dyno,player/dyno",
}
launched = call("ngfx_launch_session", {"mode": "launch", "activity": "trace", "attachPolicy": "auto", "exe": PROJECT + "/target/release/incus_artus.exe", "cwd": PROJECT, "args": [], "env": env, "platform": "windows", "outputDir": out})
print("launched:", json.dumps(launched)[:1500])
session = launched.get("sessionId") or launched.get("session", {}).get("sessionId")
if not session:
    print("no session id")
    sys.exit(1)

# The game's own log: wait for the line.
started = time.time()
log = None
while time.time() - started < 240:
    new = set(glob.glob(os.path.join(PROJECT, "logs", "session-*.log"))) - logs_before
    if new:
        log = max(new, key=os.path.getmtime)
        if line in open(log, encoding="utf-8", errors="replace").read():
            break
    time.sleep(0.5)
else:
    print("never saw", repr(line), "in", log)
    print(json.dumps(call("ngfx_get_session_status", {"sessionId": session}))[:2000])
    sys.exit(1)
print(f"saw {line!r} after {time.time() - started:.1f} s; tracing in {delay} s")
time.sleep(delay)
scopes = [
    {"mode": "frameLimited", "startAfter": {"type": "manualTrigger"}, "limitToFrames": 3, "maxDurationMs": 1000},
    {"mode": "timeLimited", "maxDurationMs": 500},
    {"mode": "frameLimited", "startAfter": {"type": "frameCount", "count": 1}, "limitToFrames": 1},
    {"mode": "submitLimited", "limitToSubmits": 10},
]
trace = None
for k, scope in enumerate(scopes):
    reply = mcp.request("tools/call", {"name": "ngfx_take_gpu_trace", "arguments": {"acquisitionId": f"{name}-{k}-{int(time.time())}", "sessionId": session, "traceScope": scope, "completionAction": "none"}})
    result = (reply or {}).get("result", {})
    if (reply or {}).get("error") or result.get("isError"):
        print(f"scope {k} {json.dumps(scope)} FAILED:", json.dumps(reply)[:1500])
        continue
    text = "\n".join(item.get("text", "") for item in result.get("content", []) if item.get("type") == "text")
    trace = result.get("structuredContent") or (json.loads(text) if text.strip().startswith("{") else text)
    print(f"scope {k} worked:", json.dumps(trace)[:3000])
    break
if trace is None:
    print(json.dumps(call("ngfx_get_session_status", {"sessionId": session}))[:1500])
    sys.exit(1)
artifact = trace.get("artifact", {}).get("artifactId") if isinstance(trace, dict) else None
if artifact:
    open(os.path.join(out, "artifact.txt"), "w").write(artifact)
    described = call("ngfx_describe_artifact", {"artifactId": artifact})
    open(os.path.join(out, "describe.json"), "w").write(json.dumps(described, indent=1))
    print("described:", json.dumps(described)[:4000])
print(json.dumps(call("ngfx_get_session_status", {"sessionId": session}))[:1500])

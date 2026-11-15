"""A shader's hot instructions, traced back to the shader text the card's compiler was given.

Nsight names a hot instruction by its line in the compiled program's listing (from "target datalayout" on). The same
text compiled again with debug marks carries, instruction for instruction, the line of the shader text each came from.

python trace_back.py LISTING DEBUG_LISTING HLSL SHADER TRACE_FOLDER... [--offset=108] [--top=40]"""
import io
import json
import os
import re
import sys

args = [a for a in sys.argv[1:] if not a.startswith("--")]
opts = dict(a[2:].split("=", 1) for a in sys.argv[1:] if a.startswith("--"))
listing, debug_listing, hlsl_path, shader = args[0], args[1], args[2], args[3]
folders = args[4:]
offset = int(opts.get("offset", 108))
top = int(opts.get("top", 40))


def read(path):
    return io.open(path, encoding="utf-8", errors="replace").read().replace("\r\n", "\n").split("\n")


def instructions(lines):
    """(line index, text) of each instruction of the one function, in order; debug calls left out."""
    out, inside = [], False
    for i, line in enumerate(lines):
        if line.startswith("define "):
            inside = True
            continue
        if inside and line.startswith("}"):
            break
        if not inside or not line.startswith("  ") or line.strip().startswith(";"):
            continue
        if "@llvm.dbg." in line:
            continue
        out.append((i, line))
    return out


def opcode(text):
    t = text.strip()
    t = re.sub(r"^%[\w.\-$]+ = ", "", t)
    word = t.split(" ", 1)[0]
    if word in ("call", "tail"):
        m = re.search(r"@([\w.]+)\(", t)
        return "call " + (m.group(1) if m else "?")
    return word


plain = read(listing)
debug = read(debug_listing)
hlsl = read(hlsl_path)
a, b = instructions(plain), instructions(debug)
same = len(a) == len(b) and all(opcode(x[1]) == opcode(y[1]) for x, y in zip(a, b))
print("instructions: %d in the game's listing, %d in the one with debug marks; the same in order: %s" % (len(a), len(b), same))
if not same:
    for k, (x, y) in enumerate(zip(a, b)):
        if opcode(x[1]) != opcode(y[1]):
            print("  first difference at instruction %d:\n    %s\n    %s" % (k, x[1][:140], y[1][:140]))
            break
by_line = {i: k for k, (i, _) in enumerate(a)}


def function_of(n):
    """The function of the shader text a line is in: the nearest line above that starts a function."""
    for i in range(n - 1, -1, -1):
        m = re.match(r"^[\w<>\[\]]+ (\w+)\(.*\)\s*$", hlsl[i]) or re.match(r"^[\w<>\[\]]+ (\w+)\(.*$", hlsl[i])
        if m and not hlsl[i].startswith(" "):
            return m.group(1)
    return "?"


def where(k):
    text = b[k][1]
    m = re.search(r"; line:(\d+) col:(\d+)", text)
    if not m:
        return None, None
    return int(m.group(1)), int(m.group(2))


def reads_before(k, span=8):
    """The nearest read of a buffer or a texture at or before instruction k, with the line it came from."""
    for j in range(k, max(k - span, -1), -1):
        t = b[j][1]
        m = re.search(r"@dx\.op\.(rawBufferLoad|bufferLoad|textureLoad|sampleLevel|sample|cbufferLoadLegacy)\.", t)
        if m:
            handle = re.search(r"%dx\.types\.Handle (%[\w.\-$]+)", t)
            src = None
            if handle:
                # The handle's own definition names the resource.
                for jj in range(j, -1, -1):
                    if b[jj][1].strip().startswith(handle.group(1) + " = "):
                        src = re.search(r"%dx\.types\.Handle (%[\w.\-$]+)", b[jj][1])
                        src = src.group(1) if src else b[jj][1].strip()[:60]
                        break
            return k - j, m.group(1), src, where(j)[0]
    return None


for folder in folders:
    data = json.load(open(os.path.join(folder, "shader_profiler_hotspots.json"), encoding="utf-8"))
    cols = data["columns"]
    rows = [dict(zip(cols, r)) for r in data["rows"]]
    rows = [r for r in rows if r.get("shader") == shader and r.get("sourceType") == "intermediate"]
    shaders = json.load(open(os.path.join(folder, "shader_profiler.json"), encoding="utf-8"))
    scols = shaders["columns"]
    whole = [dict(zip(scols, r)) for r in shaders["rows"]]
    whole = next((s["samples"] for s in whole if s.get("shader") == shader), 0)
    listed = sum(r["samples"] for r in rows)
    rows.sort(key=lambda r: -r["samples"])
    print("\n===== %s: %s, %d samples; its %d hottest instructions hold %d (%.0f%%)" % (os.path.basename(folder), shader, whole, len(rows), listed, 100.0 * listed / max(whole, 1)))
    for r in rows[:top]:
        n = r["sourceLine"]
        i = n - 1 + offset
        share = 100.0 * r["samples"] / max(whole, 1)
        stalls = ",".join(r.get("dominantStalls") or [])
        if n <= 1:
            print("%5.1f%%  %-40s (no line given)" % (share, stalls[:40]))
            continue
        # A line that isn't an instruction (a label, a blank): the next instruction below it.
        j = i
        while j < len(plain) and j not in by_line:
            j += 1
        if j not in by_line:
            print("%5.1f%%  %-40s line %d: past the end" % (share, stalls[:40], n))
            continue
        k = by_line[j]
        line, col = where(k)
        fn = function_of(line) if line else "?"
        text = hlsl[line - 1].strip() if line else "(no mark)"
        read_ = reads_before(k)
        note = ""
        if read_:
            back, kind, src, rline = read_
            note = "   <- %s of %s%s" % (kind, src, "" if back == 0 else " (%d before, text line %s)" % (back, rline))
        print("%5.1f%%  %-40s %-6s %-22s %4s: %s%s" % (share, stalls[:40], opcode(plain[j]).replace("call ", "")[:6], fn[:22], line, text[:110], note[:120]))

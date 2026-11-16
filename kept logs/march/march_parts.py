"""The far march's compiled program, by its parts: how many of its instructions each part is, as a tree, and
what the march carries round its loop.

The card's compiler is given the march as HLSL text (naga's, from `far.wgsl` and `far_all.wgsl`). Compiled again
with debug marks, every instruction names the line of that text it came from, and for a function drawn into
another, the line it was called from, all the way out. So each instruction falls in exactly one place of the tree
and the parts add up to the whole.

    python march_parts.py FOLDER [march_all.compute.hlsl] [--entry=march_all] [--min=12]

FOLDER holds the text (kept by the picture test with FAR_TEST_SHADER_TEXT=FOLDER, or cut out of a session's log
made with INCUS_SHADER_TEXT=1 by `kept logs/nsight/b175/scripts/hlsl_from_log.py`). Writes, beside the text:
`<name>.ll` (the program as the game's compiler makes it), `<name>.debug.ll` (the same with debug marks), and
prints the tree and the carried values.

What the numbers are, and aren't: instructions of the program as it leaves DXC (DXIL), which is what the card's
driver then compiles. They say how much program a part is. They don't say how long it takes (a part most steps
never reach can be large), nor how many registers it holds (only Nsight's own window shows a program's registers).
"""
import io
import os
import re
import subprocess
import sys

DXC = r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\dxc.exe"
# As wgpu's DX12 backend asks DXC for it (`kept logs/nsight/b175`'s notes: cs_6_7, -HV 2018 -no-warnings -Ges).
FLAGS = ["-T", "cs_6_7", "-HV", "2018", "-no-warnings", "-Ges"]

args = [a for a in sys.argv[1:] if not a.startswith("--")]
opts = dict(a[2:].split("=", 1) for a in sys.argv[1:] if a.startswith("--"))
folder = args[0]
name = args[1] if len(args) > 1 else "march_all.compute.hlsl"
entry = opts.get("entry", "march_all")
least = int(opts.get("min", 12))
hlsl_path = os.path.join(folder, name)
stem = os.path.splitext(hlsl_path)[0]

# The march's own blocks: a block begins at the first line (after the block before) that matches, `back` lines
# earlier. Patterns are naga's text for `far.wgsl`'s march; if the march is rewritten, these want another look
# (a block that isn't found is said, and its lines fall to the block before).
MARCH_BLOCKS = [
    ("its start: what a ray sets out with", r"^Seen march\(", 0),
    ("each step: its stops (the ray's end, covered, a cut's stop, the world's floor)", r"^while\s*\(true\)", 0),
    ("each step: the level for this distance", r"= lv_from;", 1),
    ("each step: its region, summed or not", r"^keep_place\(", 0),
    ("the skip over clear air (the height table's three rows)", r"^skipped = false;", 0),
    ("the hand-over: how much of the next level to mix in", r"^over = 0\.0;", 0),
    ("the look: the hand-over's two reads and their mix", r"spot_kept\(kept,", 3),
    ("the look: its own level's read", r"far_blend_kept\(", 3),
    ("the step's length: surface, thin haze or air; the walk back in", r"^float size_\d+ = float\(\(int\(1\) << ", 1),
    ("the gather: what covers the step (bulk, water, faces)", r"^float length_ = ", 2),
    ("the gather: its light and colour", r"^touched = min\(", 2),
    ("the gather: the floor's cells kept", r"\.z <= \(size_\d+ \* 1\.5\)", 0),
    ("the gather: the first surface, and what's still let through", r"^if \(\(_e\d+ > 0\.5\)\) \{", 1),
    ("the step's end: this look kept for the next", r"^before = _e\d+;", 1),
    ("after the last step: the floor's ground, the sky behind, what's handed back", r"= floored;", 0),
]


def run(extra, out):
    done = subprocess.run([DXC, "-E", entry] + FLAGS + extra + ["-Fc", out, hlsl_path], capture_output=True, text=True)
    if done.returncode != 0:
        sys.exit("DXC refused the text:\n" + done.stdout + done.stderr)


def read(path):
    return io.open(path, encoding="utf-8", errors="replace").read().replace("\r\n", "\n").split("\n")


def body(lines):
    """The entry function's lines, with each instruction's block."""
    out, inside, block = [], False, "entry"
    for line in lines:
        if line.startswith("define ") and "@" + entry + "(" in line:
            inside = True
            continue
        if not inside:
            continue
        if line.startswith("}"):
            break
        # A block begins at its label: a number in the game's listing, often a name in the one with debug marks.
        label = re.match(r"^(?:; <label>:(\d+)|([^\s;][^:]*):)\s*(?:;.*)?$", line)
        if label:
            block = label.group(1) or label.group(2)
            continue
        if not line.startswith("  ") or line.strip().startswith(";"):
            continue
        # A switch's cases and its closing bracket are lines of the listing, not instructions.
        if line.strip().startswith("]") or re.match(r"^\s+i\d+ -?\d+, label ", line):
            continue
        out.append((block, line))
    return out


def opcode(text):
    t = re.sub(r"^%[\w.\-$]+ = ", "", text.strip())
    word = t.split(" ", 1)[0]
    if word in ("call", "tail"):
        m = re.search(r"@([\w.]+)\(", t)
        return "call " + (m.group(1) if m else "?")
    return word


run([], stem + ".ll")
run(["-Zi"], stem + ".debug.ll")
plain = [(b, l) for b, l in body(read(stem + ".ll"))]
debug_all = body(read(stem + ".debug.ll"))
debug = [(b, l) for b, l in debug_all if "@llvm.dbg." not in l]
same = len(plain) == len(debug) and all(opcode(x[1]) == opcode(y[1]) for x, y in zip(plain, debug))
hlsl = read(hlsl_path)
print("%s: %d instructions as the game's compiler makes it; with debug marks %d, the same in order: %s" % (name, len(plain), len(debug), same))
if not same:
    sys.exit("The program with debug marks isn't the game's: nothing below would be true of it.")

# The debug marks: where each place is, and what it's inside.
meta = {}
for line in read(stem + ".debug.ll"):
    m = re.match(r"^!(\d+) = (?:distinct )?!(\w+)\((.*)\)\s*$", line)
    if m:
        meta[int(m.group(1))] = (m.group(2), m.group(3))


def field(text, key):
    m = re.search(r"\b" + key + r": (!?\"[^\"]*\"|!?[\w.]+)", text)
    return m.group(1) if m else None


def function_of(scope):
    """The function a scope lies in."""
    seen = 0
    while scope is not None and seen < 64:
        kind, text = meta.get(scope, ("", ""))
        if kind == "DISubprogram":
            return (field(text, "name") or "?").strip('"')
        up = field(text, "scope")
        scope = int(up[1:]) if up and up.startswith("!") else None
        seen += 1
    return "?"


def frames(place):
    """A place's frames, outermost first: (function, line)."""
    out = []
    while place is not None:
        kind, text = meta.get(place, ("", ""))
        if kind != "DILocation":
            break
        scope = field(text, "scope")
        out.append((function_of(int(scope[1:])) if scope else "?", int(field(text, "line") or 0)))
        up = field(text, "inlinedAt")
        place = int(up[1:]) if up else None
    return out[::-1]


# The march's blocks, by line of the text.
start = next((i for i, l in enumerate(hlsl) if re.match(MARCH_BLOCKS[0][1], l)), None)
block_at = []
if start is not None:
    at = start
    for label, pattern, back in MARCH_BLOCKS:
        found = next((i for i in range(at, len(hlsl)) if re.search(pattern, hlsl[i].strip()) or re.search(pattern, hlsl[i])), None)
        if found is None:
            print("  (the march's block isn't found in the text: %s)" % label)
            continue
        at = found
        block_at.append((max(found - back, start), label))


def march_block(line):
    label = "?"
    for first, name_ in block_at:
        if line - 1 >= first:
            label = name_
    return label


tree = {}
no_line = 0
for _, line in debug:
    m = re.search(r"!dbg !(\d+)", line)
    path = []
    if m:
        for depth, (function, at) in enumerate(frames(int(m.group(1)))):
            if depth == 0 or function != path[-1].split(": ")[0]:
                path.append(function)
            if function == "march":
                path.append("march: " + march_block(at))
    if not path:
        no_line += 1
        path = [entry, "(joins, where values from different paths meet; no line of their own)"]
    node = tree
    for part in path:
        node = node.setdefault(part, {"": 0})
        node[""] += 1


def show(node, label, depth, whole):
    count = node[""]
    kids = sorted(((k, v) for k, v in node.items() if k), key=lambda kv: -kv[1][""])
    own = count - sum(v[""] for _, v in kids)
    bar = "#" * max(1, round(40.0 * count / whole)) if count * 200 >= whole else ""
    print("%s%-*s %5d  %5.1f%%  %s" % ("  " * depth, max(1, 78 - 2 * depth), label.split(": ", 1)[-1][: 78 - 2 * depth], count, 100.0 * count / whole, bar))
    small = 0
    if kids and own:
        kids.append(("(its own lines)", {"": own}))
        kids.sort(key=lambda kv: -kv[1][""])
    for k, v in kids:
        if v[""] < least:
            small += v[""]
            continue
        show(v, k, depth + 1, whole)
    if small:
        print("%s%-*s %5d  %5.1f%%" % ("  " * (depth + 1), max(1, 78 - 2 * (depth + 1)), "(smaller parts together)", small, 100.0 * small / whole))


print("\nThe program by its parts (instructions; share of the whole):\n")
for k, v in tree.items():
    show(v, k, 0, len(debug))

# What a loop carries round: the values at its head that come back from its own end (a phi each, a number each).
# The march's loop is the block with the most of them. A value's name is its own, or (the head's values are often
# given no name of their own) the name of a value it's fed from.
names = {}
for _, line in debug_all:
    m = re.search(r"@llvm\.dbg\.value\(metadata [^%]*(%[\w.]+),.*; var:\"([^\"]+)\".*func:\"([^\"]+)\"", line)
    if m:
        names.setdefault(m.group(1), (m.group(2), m.group(3)))
phis, fed = {}, {}
for block, line in debug:
    m = re.match(r"^\s*(%[\w.]+) = phi (\S+) (.*)$", line)
    if m:
        phis.setdefault(block, []).append(m.group(1))
        fed[m.group(1)] = re.findall(r"\[ (%[\w.]+),", m.group(3))


def name_of(value, depth=0):
    if value in names:
        return names[value]
    if depth < 4:
        for source in fed.get(value, []):
            found = name_of(source, depth + 1)
            if found:
                return found
    return None


print("\nWhat the march carries round its loop (the block where the most values meet: each is a number held for")
print("every ray, every step, whether the step uses it or not):")
for block, values in sorted(phis.items(), key=lambda kv: -len(kv[1]))[:1]:
    held = {}
    for value in values:
        var = name_of(value) or ("(no name)", "")
        key = var[0] if not var[1] or var[1] == "march" else "%s (%s)" % var
        held[key] = held.get(key, 0) + 1
    print("\n  %d values" % len(values))
    for key, n in sorted(held.items(), key=lambda kv: (-kv[1], kv[0])):
        print("    %3d  %s" % (n, key))

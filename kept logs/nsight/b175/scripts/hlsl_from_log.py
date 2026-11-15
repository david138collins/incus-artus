"""Each shader's text as the card's compiler was given it, out of a session's log (INCUS_SHADER_TEXT=1):
python hlsl_from_log.py LOG OUT_FOLDER"""
import io
import os
import re
import sys

log, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)
text = io.open(log, encoding="utf-8", errors="replace").read().replace("\r\n", "\n")
# A log line begins "   12.345 s  LEVEL target: "; a shader's text runs from its heading to the next such line.
line_start = re.compile(r"^ *\d+\.\d{3} s  [A-Z]+ +\S+: ", re.M)
heading = re.compile(r'^ *\d+\.\d{3} s  DEBUG +wgpu_hal::dx12::device: Naga generated shader for "([^"]+)" at (\w+):\n', re.M)
seen = {}
for found in heading.finditer(text):
    name, stage = found.group(1), found.group(2)
    after = line_start.search(text, found.end())
    source = text[found.end(): after.start() if after else len(text)]
    # The log's own line ending follows the text.
    if source.endswith("\n"):
        source = source[:-1]
    n = seen.get(name, 0)
    seen[name] = n + 1
    path = os.path.join(out, "%s%s.%s.hlsl" % (name, "" if n == 0 else "_%d" % n, stage.lower()))
    io.open(path, "w", encoding="utf-8", newline="\n").write(source)
    print("%-28s %-9s %7d lines  %s" % (name, stage, source.count("\n") + 1, os.path.basename(path)))

"""The one-cell blend probe split in two: P7c for the sun's and the sky's rays alone (the far march keeps the eight-cell
blend), P7m for the far march alone (the rays keep theirs). Each says how much of the one-cell blend's 41% is its."""
import os

S = os.path.dirname(os.path.abspath(__file__))
src = open(r"C:\Users\David\Desktop\Incus Artus\src\sight\march.wgsl", encoding="utf-8").read()

one = """// The probe's one-cell read: the cell a point is in, in place of the eight round it (the picture blocky).
fn blend_one(level: i32, p: vec3<f32>) -> Look {
    let size = f32(1 << u32(level));
    let c = vec3<i32>(floor(p / size));
    let entry = entry_at(level, c.x, c.y);
    if (entry.y == NOT_BAKED) {
        return empty_look(false);
    }
    var out = from_band(entry.x, entry.y, c.z);
    if (out.surface <= 0.0) {
        out.colour = ROCK;
    }
    return out;
}

// Where in a level's stretch (0 its start, 1 where the next takes over) the blend with the next begins."""
anchor = "// Where in a level's stretch (0 its start, 1 where the next takes over) the blend with the next begins."
assert src.count(anchor) == 1
base = src.replace(anchor, one)

cones_old = """        var look = blend(level, q);
        // A live level not made here yet: the bake's first, there wherever its region is.
        if (!look.known && level < header[0]) {
            level = header[0];
            look = blend(level, q);
        }"""
cones_new = cones_old.replace("blend(level, q)", "blend_one(level, q)")
assert base.count(cones_old) == 1
p7c = base.replace(cones_old, cones_new)
open(os.path.join(S, "march_p7c_cones_one.wgsl"), "w", encoding="utf-8", newline="").write(p7c)

march_old = [
    ("        var look = blend(level, p);\n", "        var look = blend_one(level, p);\n"),
    ("            look = blend(level, p);\n", "            look = blend_one(level, p);\n"),
    ("            let next = blend(kept(level + 1), p);\n", "            let next = blend_one(kept(level + 1), p);\n"),
]
p7m = base
for old, new in march_old:
    assert p7m.count(old) == 1, old
    p7m = p7m.replace(old, new)
open(os.path.join(S, "march_p7m_march_one.wgsl"), "w", encoding="utf-8", newline="").write(p7m)
print("ok: blend calls left in P7c's march", p7c.count("blend(level, p)"), "and in P7m's rays", p7m.count("blend(level, q)"))

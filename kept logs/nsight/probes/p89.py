"""Two probes to size the march's split before building it: each half of `march_hit` alone, so a trace taken with it
as the twin shows that half's registers beside the game's whole march (and its flight, what that half costs).

P8, the far march alone: the near walk never run (rays march the cells from where they start, the near blocks
clouds). P9, the near walk alone: past the blocks' reach nothing more (the sky)."""
import os

S = os.path.dirname(os.path.abspath(__file__))
src = open(r"C:\Users\David\Desktop\Incus Artus\src\sight\march.wgsl", encoding="utf-8").read()

walk = """    var start_at = begin;
    if (header[2] == 0 && is_kept(0)) {
        let blocks_end = min(1.0 / camera.view.z, camera.layer.w);"""
assert src.count(walk) == 1
p8 = src.replace(walk, walk.replace("if (header[2] == 0 && is_kept(0)) {", "if (false && header[2] == 0 && is_kept(0)) {"))
open(os.path.join(S, "march_p8_far_only.wgsl"), "w", encoding="utf-8", newline="").write(p8)

after_walk = """            colour = h.colour;
            sunny = h.sunny;
            glow = h.glow;
            through = h.through;
            first = h.first;
            touched = h.first;
            start_at = h.t;
        }
    }
"""
assert src.count(after_walk) == 1
p9 = src.replace(after_walk, after_walk + """    // The probe: the near walk alone, past it the sky.
    if (true) {
        let gathered_walk = colour;
        return Seen(colour + through * sky(d), first, f32(steps), select(-1, 0, first < camera.reach.x), sunny, o + d * min(first, start_at), 0.0, vec3<f32>(0.0), false, gathered_walk - sunny, touched, 0.0, vec3<f32>(0.0), glow, through);
    }
""")
open(os.path.join(S, "march_p9_near_only.wgsl"), "w", encoding="utf-8", newline="").write(p9)
print("ok")

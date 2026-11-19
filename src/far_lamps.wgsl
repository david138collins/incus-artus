// ── Seeing the far lamps (build 223) ──
//
// Not a shader on its own: `far_eye.rs` puts this after `far.wgsl` (the whole march), so a lamp is seen as the far
// picture would see it. For each lamp in the far lamps' list (`near.rs`, `LampsLane`) past where the far rays begin,
// one ray marched from the eye towards it, with the finest far layer's camera (its cells the finest the far eye reads
// at that distance), to a little short of it; seen if the ray met no surface on the way. [David, 2026-11-18, of
// build 221: "the single pixels of light don't twinkle quite the way they did in old incus, from a distance"; old's
// far glows were seen or not by a ray marched to each, `old/src/sight/march.wgsl`'s `glow_pass`. Build 223 tried the
// far cells' column tops and then the far picture's own distances first: the tops' columns were too coarse, and the
// picture's distances, where its ray first touched anything, too near (a cave mouth's soft fringe, an 8 x 8 ray's
// ridge) to tell a lamp at a cave's mouth from one deep inside.]

struct Sight {
    // x: how many lamps; y: where the far rays begin (blocks from the eye); z: how many far cells short of a lamp the
    // ray stops (the hill a cave mouth is cut into, the lamp's own cell).
    numbers: vec4<f32>,
    // In turns (build 226): x: this frame's turn; y: how many turns the list is seen in (1: every lamp this frame).
    // Thread j sees to lamp j * turns + turn; the others keep the answer of their own turn.
    turns: vec4<f32>,
}

@group(1) @binding(0) var<uniform> sight: Sight;
// The lamps: each its cell (north, east, up; the world's), and its light and voice in the low 16 bits of the fourth
// word; over them (`1 << 16`) whether it's seen, written here: seen, or nearer than where the far rays begin (which
// the near picture decides). (In the lamps' own words: the march's program has room for one more buffer, no more.)
@group(1) @binding(1) var<storage, read_write> sight_lamps: array<vec4<i32>>;

@compute @workgroup_size(64)
fn lamp_sight(@builtin(global_invocation_id) id: vec3<u32>) {
    let k = id.x * u32(sight.turns.y) + u32(sight.turns.x);
    if (k >= u32(sight.numbers.x)) {
        return;
    }
    let to = vec3<f32>(sight_lamps[k].xyz) + 0.5 - camera.position.xyz;
    let off = length(to);
    let d = to / max(off, 0.001);
    let end = off - sight.numbers.z * exp2(f32(level_at(off))) - 2.0;
    let kept = sight_lamps[k].w & 0xFFFF;
    // (A lamp behind the eye keeps its answer: the half of the world ahead is far wider than the view, so its turn
    // comes round again before it can be on the screen.)
    if (dot(d, camera.forward.xyz) <= 0.0) {
        return;
    }
    if (end <= sight.numbers.y) {
        sight_lamps[k].w = kept | (1 << 16u);
        return;
    }
    let seen = march(camera.position.xyz, d, 0.0, sight.numbers.y, end, 1e9, 1.0, false);
    sight_lamps[k].w = kept | (select(0, 1, seen.distance >= end) << 16u);
}

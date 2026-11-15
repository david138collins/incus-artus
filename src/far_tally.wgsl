// ── The longest ray (the Oracle's step 0): a far layer's rays' steps counted ──
//
// A pass of its own, run once a second on a flight: each of the layer's pixels' steps (kept beside its distance by the
// march) into its counts: the most any ray took, and how many took over 256, 384 and 512; then, for the cut and for
// knowing what each layer marches: how many rays took `long` steps or more, how many of those are in the long rays'
// list (their place in it is kept beside their steps), how many rays marched at all, and all their steps summed; and
// of the cut itself: the longest chain (the most steps any one thread took: a cut ray's longest length, kept beside
// its steps), how many rays were cut, and how many are in the list at all.

@group(0) @binding(0) var steps: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> counts: array<atomic<u32>, 12>;
// The layer's part of the texture: where it starts (x, y) and its size (the one dispatch keeps the layers' distances
// side by side in one texture; a layer's own texture is all its part).
// And how many steps make a ray long (`long.x`).
struct Part {
    part: vec4<u32>,
    long: vec4<u32>,
}
@group(0) @binding(2) var<uniform> part: Part;

@compute @workgroup_size(8, 8)
fn tally_pass(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= part.part.z || id.y >= part.part.w) {
        return;
    }
    let kept = textureLoad(steps, vec2<i32>(id.xy + part.part.xy), 0);
    let n = u32(kept.y);
    atomicMax(&counts[0], n);
    if (n > 256u) {
        atomicAdd(&counts[1], 1u);
    }
    if (n > 384u) {
        atomicAdd(&counts[2], 1u);
    }
    if (n > 512u) {
        atomicAdd(&counts[3], 1u);
    }
    if (n >= part.long.x) {
        atomicAdd(&counts[4], 1u);
        if (kept.w > 0.0) {
            atomicAdd(&counts[5], 1u);
        }
    }
    if (n > 0u) {
        atomicAdd(&counts[6], 1u);
        atomicAdd(&counts[7], n);
        atomicMax(&counts[8], select(n, u32(kept.z), kept.z > 0.5));
    }
    if (kept.z > 0.5) {
        atomicAdd(&counts[9], 1u);
    }
    if (kept.w > 0.5) {
        atomicAdd(&counts[10], 1u);
    }
}

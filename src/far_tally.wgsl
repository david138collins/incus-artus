// ── The longest ray (the Oracle's step 0): a far layer's rays' steps counted ──
//
// A pass of its own, run once a second on a flight: each of the layer's pixels' steps (kept beside its distance by the
// march) into four counts: the most any ray took, and how many took over 256, 384 and 512.

@group(0) @binding(0) var steps: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> counts: array<atomic<u32>, 4>;

@compute @workgroup_size(8, 8)
fn tally_pass(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(steps);
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }
    let n = u32(textureLoad(steps, vec2<i32>(id.xy), 0).y);
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
}

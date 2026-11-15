// ── Threads kept waiting (build 184): the march's grid counted group by group ──
//
// A pass of its own, run with the rays' count on a flight (`far_tally.wgsl`), outside the passes' timing. The card
// runs the march's threads in groups that go on as long as their longest thread, the rest waiting. For every 8 x 8
// group of the march's grid this adds up the steps its threads took (busy) and what the group was held for (its
// longest thread's steps times its threads), in two parts: the layers' threads, and the strip's (the cut rays'
// lengths, `far_all.wgsl`). Held is counted two ways, as it isn't known how the card packs a group: as two groups of
// 32 (its first four rows, its last four), and as one of 64. And how many threads marched at all (each pays its work
// before a first step, which no count here holds).
//
// A thread's steps: a whole ray's, kept beside its distance; a listed ray's own thread, its first length's; a thread
// of the strip, its length's (both from the lengths' stacks). [Build 183's flight: the same steps cut and uncut, and the
// march a third longer where the card works. The reading this counts for: threads kept waiting.]

// This frame's distances (beside each: its ray's steps, a cut ray's longest length, its place in this frame's list)
// and the frame before's (beside each: its place in the frame before's list, which is the place its lengths' stacks
// are kept at this frame).
@group(0) @binding(0) var far_now: texture_2d<f32>;
@group(0) @binding(1) var far_before: texture_2d<f32>;
// The cut's store: the lists, then the lengths' stacks.
@group(0) @binding(2) var<storage, read> cut: array<u32>;

struct Numbers {
    // Each layer: its size (x, y), where its distances start in the texture (z), where its threads start in the
    // march's grid (w).
    layers: array<vec4<u32>, 3>,
    // The strip: x where its threads start in the grid, y its threads across (0: no ray is cut this frame), z how
    // many rays a list holds, w how many lengths a list of lengths holds.
    strip: vec4<u32>,
    // x a stack's words, zw the grid's groups across and down.
    more: vec4<u32>,
    // Where things are in `cut` (`far_all.wgsl`): x the frame before's list of rays, y the frame before's list of
    // lengths, z the rays' own stacks, w the lengths' stacks.
    at: vec4<u32>,
}
@group(0) @binding(3) var<uniform> numbers: Numbers;
// The layers': steps taken, held by 32s, held by 64s, threads that marched; then the strip's, the same four.
@group(0) @binding(4) var<storage, read_write> counts: array<atomic<u32>, 12>;

// The steps the length whose stack is at `stack` took this frame.
fn stack_steps(stack: u32) -> u32 {
    return (cut[stack + 9u] >> 5u) & 0x3FFFFFFu;
}

// Where the ray at `slot` of the frame before's list is in the distances' texture, if it was listed there and was
// joined from its lengths this frame (x under nothing: it wasn't).
fn listed_at(slot: u32) -> vec2<i32> {
    if (slot >= numbers.strip.z) {
        return vec2<i32>(-1);
    }
    let entry = cut[numbers.at.x + slot * 3u];
    let layer = entry >> 30u;
    let pixel = vec2<u32>(entry & 0x7FFFu, (entry >> 15u) & 0x7FFFu);
    if (layer == 3u || any(pixel >= numbers.layers[layer].xy)) {
        return vec2<i32>(-1);
    }
    let at = vec2<i32>(i32(pixel.x + numbers.layers[layer].z), i32(pixel.y));
    if (u32(textureLoad(far_before, at, 0).w) != slot + 1u || textureLoad(far_now, at, 0).z < 0.5) {
        return vec2<i32>(-1);
    }
    return at;
}

// The steps the march's thread at `id` of its grid took.
fn thread_steps(id: vec2<u32>) -> u32 {
    if (id.x >= numbers.strip.x) {
        // The strip: a thread a place in the frame before's list of lengths, row after row. A place counts if its
        // ray's cuts say it's one of its lengths'.
        if (numbers.strip.y == 0u || id.x >= numbers.strip.x + numbers.strip.y) {
            return 0u;
        }
        let piece = id.y * numbers.strip.y + (id.x - numbers.strip.x);
        if (piece >= numbers.strip.w) {
            return 0u;
        }
        let slot = cut[numbers.at.y + piece];
        if (listed_at(slot).x < 0) {
            return 0u;
        }
        let cuts = cut[numbers.at.x + slot * 3u + 1u];
        let first = cut[numbers.at.x + slot * 3u + 2u];
        if (piece < first || piece - first >= countOneBits(cuts)) {
            return 0u;
        }
        return stack_steps(numbers.at.w + piece * numbers.more.x);
    }
    var layer = 0u;
    if (id.x >= numbers.layers[2].w) {
        layer = 2u;
    } else if (id.x >= numbers.layers[1].w) {
        layer = 1u;
    }
    let pixel = vec2<u32>(id.x - numbers.layers[layer].w, id.y);
    if (any(pixel >= numbers.layers[layer].xy)) {
        return 0u;
    }
    let at = vec2<i32>(i32(pixel.x + numbers.layers[layer].z), i32(pixel.y));
    let kept = textureLoad(far_now, at, 0);
    if (kept.z > 0.5) {
        // A listed ray: its own thread marched its first length.
        let listed = textureLoad(far_before, at, 0).w;
        if (listed < 0.5) {
            return 0u;
        }
        return stack_steps(numbers.at.z + (u32(listed) - 1u) * numbers.more.x);
    }
    return u32(kept.y);
}

// A thread of this pass counts one 8 x 8 group of the march's grid.
@compute @workgroup_size(8, 8)
fn waiting_pass(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= numbers.more.z || id.y >= numbers.more.w) {
        return;
    }
    var busy = 0u;
    var marched = 0u;
    var most_first = 0u;
    var most_last = 0u;
    for (var row = 0u; row < 8u; row++) {
        for (var column = 0u; column < 8u; column++) {
            let steps = thread_steps(id.xy * 8u + vec2<u32>(column, row));
            busy += steps;
            marched += u32(steps > 0u);
            if (row < 4u) {
                most_first = max(most_first, steps);
            } else {
                most_last = max(most_last, steps);
            }
        }
    }
    if (busy == 0u) {
        return;
    }
    let part = select(0u, 4u, id.x * 8u >= numbers.strip.x);
    atomicAdd(&counts[part], busy);
    atomicAdd(&counts[part + 1u], 32u * (most_first + most_last));
    atomicAdd(&counts[part + 2u], 64u * max(most_first, most_last));
    atomicAdd(&counts[part + 3u], marched);
}

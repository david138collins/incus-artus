// ── The far boxes (plan 2.2, change 2): rays begun at drawn boxes ──
//
// Read after `far.wgsl` (its camera, its levels' table, its bands and `level_at`). Each 256-block column of land is
// drawn as a box by the triangle hardware, at the 8 × 8 layer's size and with its rays (a layer pixel's middle), from
// under the world's floor to its band's top, swollen by what a blend can reach at its far corner: a whole cell every
// way of the level after the one read there (the hand-over mixes it in), its top rounded up to that cell. Each pixel
// keeps the farthest surface its ray passes, front or back (the depth test keeps the farthest), as a share of twice the
// reach. Past it nothing can be met: the march stops there, and a pixel with no box is sky at once.

// The level whose columns are drawn: cells of 256.
const BOX_LEVEL: i32 = 8;

struct BoxCorner {
    @builtin(position) clip: vec4<f32>,
    // The corner from the eye (blocks), carried to each pixel for its distance.
    @location(0) rel: vec3<f32>,
}

@vertex
fn box_corner(@builtin(vertex_index) v: u32, @builtin(instance_index) i: u32) -> BoxCorner {
    // A box's 36 corners, two triangles a face, by which of its eight they are (bit 0 its north side, 1 its east, 2 its
    // top); front or back, all are drawn.
    var ids = array<u32, 36>(0u, 1u, 3u, 0u, 3u, 2u, 4u, 6u, 7u, 4u, 7u, 5u, 0u, 4u, 5u, 0u, 5u, 1u, 2u, 3u, 7u, 2u, 7u, 6u, 0u, 2u, 6u, 0u, 6u, 4u, 1u, 5u, 7u, 1u, 7u, 3u);
    var out: BoxCorner;
    // Not drawn (outside every clip, w below nothing): a column not summed.
    out.clip = vec4<f32>(0.0, 0.0, 0.0, -1.0);
    out.rel = vec3<f32>(0.0);
    let across = table(BOX_LEVEL, 3);
    let e = entries[u32(table(BOX_LEVEL, 4)) + i];
    if (e == NOT_SUMMED || e == PAST_MAP) {
        return out;
    }
    let low = bitcast<i32>(e << 16u) >> 16u;
    let count = i32(e >> 16u);
    let size = f32(1 << u32(BOX_LEVEL));
    let c = vec2<f32>(f32(table(BOX_LEVEL, 1) + i32(i) % across), f32(table(BOX_LEVEL, 2) + i32(i) / across));
    let top = f32(low + count) * size;
    let eye = camera.position.xyz;
    let lo0 = vec3<f32>(c * size, -16.0);
    let hi0 = vec3<f32>((c + 1.0) * size, top);
    let far = length(max(abs(lo0 - eye), abs(hi0 - eye)));
    let s = f32(1 << u32(min(level_at(far) + 1, TOP)));
    let lo = vec3<f32>(lo0.xy - s, lo0.z - s);
    let hi = vec3<f32>(hi0.xy + s, ceil(top / s) * s + s);
    let k = ids[v];
    let corner = select(lo, hi, vec3<bool>((k & 1u) != 0u, (k & 2u) != 0u, (k & 4u) != 0u));
    // Seen as the march's rays see it: a ray is forward plus so much of right and up (the camera's rows, scaled to the
    // view), so a point's place on the screen is its share of each over its depth along forward.
    let rel = corner - eye;
    let r = camera.right.xyz;
    let u = camera.up.xyz;
    let w = dot(rel, camera.forward.xyz);
    out.clip = vec4<f32>(dot(rel, r) / dot(r, r), dot(rel, u) / dot(u, u), 0.5 * w, w);
    out.rel = rel;
    return out;
}

@fragment
fn box_surface(corner: BoxCorner) -> @builtin(frag_depth) f32 {
    return clamp(length(corner.rel) / (2.0 * camera.reach.x), 0.0, 1.0);
}

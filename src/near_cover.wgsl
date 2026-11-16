// The near eye's cover (plan 2.3, build 201): which of the far eye's pixels the near eye's blocks wholly cover, so
// the far eye needn't march them. A thread a pixel of the far eye's base layer: it reads every screen pixel of the
// near picture under it, and writes a pixel for the base's pixel, for each of the middle layer's four under it, and
// for each of the finest's sixteen: 1 where every screen pixel there is wholly a block's (the near picture's share
// all of it: nearer than the x-fader's band, and no gap), 0 otherwise. A screen pixel past the screen's edge breaks
// no cover.

struct Sizes {
    // Each layer's size in its pixels (x, y). The base's z: its pixel's side in screen pixels (the middle layer's is
    // half that, the finest's a quarter).
    base: vec4<u32>,
    middle: vec4<u32>,
    fine: vec4<u32>,
    // The screen's size.
    screen: vec4<u32>,
}

@group(0) @binding(0) var t_near: texture_2d<f32>;
// The far eye's picture for it (`far_all.wgsl`'s `cover`): the base's pixels, the middle layer's and the finest's
// side by side from its left, as its distances are kept.
@group(0) @binding(1) var cover: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> sizes: Sizes;

@compute @workgroup_size(8, 8)
fn near_cover(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= sizes.base.x || id.y >= sizes.base.y) {
        return;
    }
    // A finest pixel's side in screen pixels.
    let side = sizes.base.z / 4u;
    let middle_at = vec2<i32>(i32(sizes.base.x), 0);
    let fine_at = vec2<i32>(i32(sizes.base.x + sizes.middle.x), 0);
    var whole = 1u;
    for (var m = 0u; m < 4u; m++) {
        let middle_pixel = id.xy * 2u + vec2<u32>(m & 1u, m >> 1u);
        var middle = 1u;
        for (var f = 0u; f < 4u; f++) {
            let fine_pixel = middle_pixel * 2u + vec2<u32>(f & 1u, f >> 1u);
            var fine = 1u;
            for (var y = 0u; y < side; y++) {
                for (var x = 0u; x < side; x++) {
                    let p = fine_pixel * side + vec2<u32>(x, y);
                    if (p.x < sizes.screen.x && p.y < sizes.screen.y && textureLoad(t_near, vec2<i32>(p), 0).a < 0.999) {
                        fine = 0u;
                    }
                }
            }
            if (fine_pixel.x < sizes.fine.x && fine_pixel.y < sizes.fine.y) {
                textureStore(cover, fine_at + vec2<i32>(fine_pixel), vec4<f32>(f32(fine), 0.0, 0.0, 1.0));
            }
            middle &= fine;
        }
        if (middle_pixel.x < sizes.middle.x && middle_pixel.y < sizes.middle.y) {
            textureStore(cover, middle_at + vec2<i32>(middle_pixel), vec4<f32>(f32(middle), 0.0, 0.0, 1.0));
        }
        whole &= middle;
    }
    textureStore(cover, vec2<i32>(id.xy), vec4<f32>(f32(whole), 0.0, 0.0, 1.0));
}

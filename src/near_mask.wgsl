// A near band's mask (build 213; `near_eye.rs`, `draw_faces`): before a band's faces are drawn, its depth is filled, as
// the nearest there is, at every pixel a nearer band's picture wholly covers. A nearer band is always in front of a
// farther one along a line of sight (bands are by true distance from the eye), so nothing the farther band could draw
// there would show; with the depth filled the card drops its faces there before their pixels' work.

// `near.wgsl`'s `Band`.
struct Band {
    picture: vec4<f32>,
    // z, w: how many of this band's pixels a side a pixel of the band just nearer than it is, and of the one before
    // that (0: no such band).
    span: vec4<f32>,
    behind: vec4<f32>,
}

@group(0) @binding(0) var<uniform> band: Band;
@group(0) @binding(1) var t_nearer_a: texture_2d<f32>;
@group(0) @binding(2) var t_nearer_b: texture_2d<f32>;

// One triangle over the whole picture.
@vertex
fn mask_whole(@builtin(vertex_index) k: u32) -> @builtin(position) vec4<f32> {
    return vec4<f32>(f32(k & 1u) * 4.0 - 1.0, f32(k >> 1u) * 4.0 - 1.0, 0.0, 1.0);
}

@fragment
fn mask_covered(@builtin(position) at: vec4<f32>) -> @builtin(frag_depth) f32 {
    let pixel = vec2<u32>(at.xy);
    var covered = false;
    if (band.span.z > 0.5) {
        covered = textureLoad(t_nearer_a, vec2<i32>(pixel / u32(band.span.z)), 0).a >= 0.999;
    }
    if (!covered && band.span.w > 0.5) {
        covered = textureLoad(t_nearer_b, vec2<i32>(pixel / u32(band.span.w)), 0).a >= 0.999;
    }
    if (!covered) {
        discard;
    }
    // (The depth is kept turned round: 1 is the nearest.)
    return 1.0;
}

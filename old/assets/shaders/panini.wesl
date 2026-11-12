// Panini projection: a full-screen pass that re-maps the normal (rectilinear) camera image.
//
// A normal camera stretches things near the screen edges, badly so at wide fields of view.
// Panini squeezes the edges back in and enlarges the middle, so a wide view keeps natural
// proportions in the centre. Based on the "Panini_Generic" method used by Unity's
// post-processing (Sharpless et al., "Pannini: A New Projection for Rendering Wide Angle
// Perspective Images").

struct FullscreenVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct PaniniPass {
    // x, y = half-size of the camera's view at distance 1 (tan of half the FOV, sideways and up),
    // z = Panini distance (0 = no effect, 1 = full), w = scale (below 1 zooms in to crop the edges).
    params: vec4<f32>,
};

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;
@group(0) @binding(2) var<uniform> panini: PaniniPass;

// Where a point on the Panini image comes from on the normal image.
fn panini_generic(view_pos: vec2<f32>, d: f32) -> vec2<f32> {
    let view_dist = 1.0 + d;
    let view_hyp_sq = view_pos.x * view_pos.x + view_dist * view_dist;
    let isect_d = view_pos.x * d;
    let isect_discrim = view_hyp_sq - isect_d * isect_d;
    let cyl_dist_minus_d = (-isect_d * view_pos.x + view_dist * sqrt(isect_discrim)) / view_hyp_sq;
    let cyl_dist = cyl_dist_minus_d + d;
    let cyl_pos = view_pos * (cyl_dist / view_dist);
    return cyl_pos / (cyl_dist - d);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let extents = panini.params.xy;
    let view_pos = (2.0 * in.uv - 1.0) * extents * panini.params.w;
    let source = panini_generic(view_pos, panini.params.z) / extents;
    let uv = source * 0.5 + 0.5;
    // Outside the rendered image (only when "crop to fit" is low): black.
    if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    return textureSample(screen_texture, screen_sampler, uv);
}

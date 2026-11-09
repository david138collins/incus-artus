// ── Objects drawn in sight (build 96, David through Counsel: "yeah, lets put the object in at the same time") ──
//
// Each placed object's model (its glTF, `models.rs`) and its sprite (a brazier's flame), drawn after the march
// into the picture it made, behind and in front of the world as its depth says (the march's touch, `upscale.wgsl`).
// The model is lit as the land round it is: the sky's light and the sun's on its faces (none of either in a cave),
// and the placed light where it stands. The sprite is full bright, turning about the upright to face you, as Doom's
// flames do.

struct View {
    // The eye, and the screen's ways (all a whole way long).
    eye: vec4<f32>,
    forward: vec4<f32>,
    right: vec4<f32>,
    up: vec4<f32>,
    // x, y: the screen's half width and half height at a block's distance; z: the nearest drawn (blocks).
    lens: vec4<f32>,
    // xyz: towards the sun; w: the sky's share of the light (`AMBIENT`).
    sun: vec4<f32>,
    // x: the sun's own light; y: the sky's (both by the time of day, as the march has them); z, w: the screen's size
    // (pixels).
    daylight: vec4<f32>,
}

@group(0) @binding(0) var<uniform> view: View;
@group(1) @binding(0) var painted: texture_2d<f32>;
@group(1) @binding(1) var pixels: sampler;

// Where a place in the world is on the screen: x and y across it, the depth the march's has (the nearest over the
// distance ahead: nearer is more), and the distance ahead.
fn on_screen(p: vec3<f32>) -> vec4<f32> {
    let r = p - view.eye.xyz;
    let ahead = dot(r, view.forward.xyz);
    return vec4<f32>(dot(r, view.right.xyz) / view.lens.x, dot(r, view.up.xyz) / view.lens.y, view.lens.z, ahead);
}

fn to_srgb(c: vec3<f32>) -> vec3<f32> {
    let low = c * 12.92;
    let high = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(high, low, c <= vec3<f32>(0.0031308));
}

// ── The models ──

struct Corner {
    // In blocks, the game's axes, from the model's own origin (its nodes placed, not yet scaled or stood on its feet).
    @location(0) at: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct Placed {
    // Its feet (blocks), and the model's own feet (the middle of its bottom, as the corners are counted).
    @location(3) feet: vec3<f32>,
    @location(4) model_feet: vec3<f32>,
    // x, y: its turn (a quarter turn each, anticlockwise seen from above), as a cosine and a sine; z: the model's
    // units to blocks; w: how much of the sky's light reaches it (0 under cover, 1 in the open).
    @location(5) turn: vec4<f32>,
    // The placed light where it stands (each colour 0 to 1, as it flickers now).
    @location(6) glow: vec3<f32>,
}

struct Shaded {
    @builtin(position) at: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) sky: f32,
    @location(3) glow: vec3<f32>,
}

fn turned(v: vec3<f32>, c: f32, s: f32) -> vec3<f32> {
    return vec3<f32>(v.x * c - v.y * s, v.x * s + v.y * c, v.z);
}

@vertex
fn model_corner(v: Corner, o: Placed) -> Shaded {
    var out: Shaded;
    let local = (v.at - o.model_feet) * o.turn.z;
    out.at = on_screen(o.feet + turned(local, o.turn.x, o.turn.y));
    out.uv = v.uv;
    out.normal = turned(v.normal, o.turn.x, o.turn.y);
    out.sky = o.turn.w;
    out.glow = o.glow;
    return out;
}

@fragment
fn model_pixel(i: Shaded, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let texel = textureSample(painted, pixels, i.uv);
    // As Blockbench's export has it: a texel see-through or solid, nothing between.
    if (texel.a < 0.5) {
        discard;
    }
    let n = normalize(select(-i.normal, i.normal, front));
    let sunny = max(dot(n, view.sun.xyz), 0.0);
    let light = i.sky * (view.sun.w * view.daylight.y + (1.0 - view.sun.w) * view.daylight.x * sunny);
    return vec4<f32>(to_srgb(clamp(texel.rgb * (light + i.glow), vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
}

// ── The sprites ──

struct Sprite {
    // Where it hangs (blocks).
    @location(0) spot: vec3<f32>,
    // x, y: its width and height (blocks); z: how far its middle is over its spot (its anchor).
    @location(1) size: vec4<f32>,
    // The frame showing now, in the strip: its left, top, right and bottom (0 to 1).
    @location(2) frame: vec4<f32>,
}

struct SpriteShaded {
    @builtin(position) at: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn sprite_corner(@builtin(vertex_index) k: u32, s: Sprite) -> SpriteShaded {
    // Two triangles: the corners' places across (0 left, 1 right) and up (0 bottom, 1 top).
    let corners = array<vec2<f32>, 6>(vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0));
    let c = corners[k];
    // Turned about the upright to face the eye: across it is the screen's right, laid flat.
    let flat = vec3<f32>(view.right.x, view.right.y, 0.0);
    let across = flat / max(length(flat), 1e-4);
    let p = s.spot + across * (c.x - 0.5) * s.size.x + vec3<f32>(0.0, 0.0, (c.y - 0.5) * s.size.y + s.size.z);
    var out: SpriteShaded;
    out.at = on_screen(p);
    out.uv = vec2<f32>(mix(s.frame.x, s.frame.z, c.x), mix(s.frame.w, s.frame.y, c.y));
    return out;
}

@fragment
fn sprite_pixel(i: SpriteShaded) -> @location(0) vec4<f32> {
    let texel = textureSample(painted, pixels, i.uv);
    if (texel.a < 0.5) {
        discard;
    }
    // Full bright: it's the light.
    return vec4<f32>(to_srgb(texel.rgb), 1.0);
}

// ── Far lights' glows (build 100) ──

struct Glow {
    // Its flame's place (blocks), and whether the eye sees it (1 or 0: `march.wgsl`'s `glow_pass`).
    @location(0) at: vec4<f32>,
    // Its light now (linear, 0 to 1 each), and how big its flame is (blocks).
    @location(1) colour: vec3<f32>,
    @location(2) size: f32,
}

struct GlowShaded {
    @builtin(position) at: vec4<f32>,
    @location(0) colour: vec3<f32>,
}

@vertex
fn glow_corner(@builtin(vertex_index) k: u32, g: Glow) -> GlowShaded {
    let corners = array<vec2<f32>, 6>(vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0));
    var out: GlowShaded;
    out.colour = vec3<f32>(0.0);
    let s = on_screen(g.at.xyz);
    // Hidden, or behind the eye: off the screen.
    if (g.at.w < 0.5 || s.w < 1.0) {
        out.at = vec4<f32>(-2.0, -2.0, 0.5, 1.0);
        return out;
    }
    let screen = view.daylight.zw;
    // Its flame across, in pixels: a square a pixel wide at the least (dimmer by as much as it's smaller), four at
    // the most, on whole pixels so it doesn't shimmer as the eye moves.
    let wide = g.size * screen.y / (2.0 * view.lens.y * s.w);
    let side = clamp(round(wide), 1.0, 4.0);
    let middle = (s.xy / s.w * vec2<f32>(0.5, -0.5) + 0.5) * screen;
    let corner = floor(middle - side * 0.5 + 0.5) + corners[k] * side;
    out.at = vec4<f32>((corner / screen - 0.5) * vec2<f32>(2.0, -2.0), 0.5, 1.0);
    let smaller = min(wide, 1.0);
    out.colour = g.colour * smaller * smaller;
    return out;
}

@fragment
fn glow_pixel(i: GlowShaded) -> @location(0) vec4<f32> {
    // Added to what's there: it's light.
    return vec4<f32>(to_srgb(clamp(i.colour, vec3<f32>(0.0), vec3<f32>(1.0))), 0.0);
}

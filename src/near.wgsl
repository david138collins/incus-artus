// The near eye's faces (plan 2.3, build 195): the chunks' merged faces drawn as triangles, lit as the far eye lights.
// `near_eye.rs` puts the far eye's own text in front of this when the program is built (`far.wgsl`'s `Camera`, its
// levels' table, its light's rule and its light store's reading), so both eyes light by one text.

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> levels: array<vec4<i32>, 48>;
@group(0) @binding(2) var t_shadow: texture_2d<f32>;
@group(0) @binding(3) var t_sky: texture_2d<f32>;
@group(0) @binding(4) var soft: sampler;
// Each block's colour from far, as light: its top's, then its sides' (the far colours' table).
@group(0) @binding(5) var<storage, read> colours: array<vec4<f32>>;
@group(0) @binding(6) var<uniform> near: Near;
// The faces' depth, for the water (its pass alone).
@group(1) @binding(0) var t_depth: texture_depth_2d;

struct Near {
    // The eye's block (north, east, up), and how far into it the eye is: a face's place is taken from the eye in whole
    // blocks first, so far from the map's middle nothing wobbles.
    cell: vec4<i32>,
    // (w: the x-fader's curve, 1 a straight fade.)
    part: vec4<f32>,
    // x: how far out the near faces reach now (blocks, level); y: the x-fader's band, how many blocks before that
    // they hand over to the far picture across; z: the nearest kept, blocks ahead of the eye (the depth is that over
    // how far ahead a place is: kept turned round, 1 the nearest and falling towards 0 far out, so it's as fine at
    // 2,000 blocks as at 2 [the common way round, to build 201, couldn't tell faces a block apart out there: a
    // hidden face won a pixel now and then, and which one differed from run to run]); w: how far ahead "nothing
    // behind" is taken to be, for the water.
    reach: vec4<f32>,
}

// How much wider than its cells a face is drawn all round, for each block of its distance from the eye (`near_corner`).
const HAIR: f32 = 0.00002;

struct Corner {
    @builtin(position) clip: vec4<f32>,
    // From the eye (north, east, up).
    @location(0) from_eye: vec3<f32>,
    // The facing (0 north, 1 south, 2 east, 3 west, 4 up, 5 down) and the block.
    @location(1) @interpolate(flat) what: vec2<u32>,
    // How far along the eye's own way.
    @location(2) ahead: f32,
}

// A face's corner. The face is three words (`near.rs`, `packed`): its place in its chunk, its facing and its width;
// its height and its block; its chunk. Four corners a face, a strip; every face is wound the same way seen from the
// side it looks out of.
@vertex
fn near_corner(@builtin(vertex_index) corner: u32, @location(0) face: vec3<u32>) -> Corner {
    let facing = (face.x >> 22u) & 7u;
    let wide = f32(((face.x >> 25u) & 31u) + 1u);
    let high = f32((face.y & 1023u) + 1u);
    let chunk = vec2<i32>(i32(face.z & 0xFFFFu) - 32768, i32(face.z >> 16u) - 32768);
    // The face's least cell, from the eye's cell, in whole blocks.
    let cell = vec3<i32>(chunk * 32 + vec2<i32>(i32(face.x & 63u), i32((face.x >> 6u) & 63u)), i32((face.x >> 12u) & 1023u)) - near.cell.xyz;
    var u = f32(corner & 1u);
    let v = f32(corner >> 1u);
    // The two ways it spreads, and the way it looks: a face that looks up or down spreads north and east; north or
    // south, east and up; east or west, north and up. On the far side of its cell if it looks the greater way.
    var spread_a = vec3<f32>(1.0, 0.0, 0.0);
    var spread_b = vec3<f32>(0.0, 1.0, 0.0);
    var out_of = vec3<f32>(0.0, 0.0, 1.0);
    if (facing < 2u) {
        spread_a = vec3<f32>(0.0, 1.0, 0.0);
        spread_b = vec3<f32>(0.0, 0.0, 1.0);
        out_of = vec3<f32>(1.0, 0.0, 0.0);
    } else if (facing < 4u) {
        spread_b = vec3<f32>(0.0, 0.0, 1.0);
        out_of = vec3<f32>(0.0, 1.0, 0.0);
    }
    // (North, west and up are wound one way as they come; the other three turned round.)
    if (facing == 1u || facing == 2u || facing == 5u) {
        u = 1.0 - u;
    }
    let greater = facing == 0u || facing == 2u || facing == 4u;
    let exact = vec3<f32>(cell) + spread_a * u * wide + spread_b * v * high + out_of * select(0.0, 1.0, greater) - near.part.xyz;
    // A hair wider than its cells all round, in its own plane, and more the farther it is (a few hundredths of a
    // pixel whatever its distance). Merged faces meet along an edge at different corners, and the card, placing
    // corners on its fine grid, can leave a pixel between them that neither covers: with the far eye's sky behind,
    // white dots that flicker as the eye moves [David, 2026-11-16, build 199: "i notice speckling white dots of sky
    // when looking down and moving"]. So neighbours overlap by that hair.
    let hair = length(exact) * HAIR;
    let from_eye = exact + spread_a * (u * 2.0 - 1.0) * hair + spread_b * (v * 2.0 - 1.0) * hair;
    // The far eye's own view: a pixel's ray is forward + x right + y up, so a place's x and y are its share of right
    // and up over how far ahead it is.
    let ahead = dot(from_eye, camera.forward.xyz);
    let x = dot(from_eye, camera.right.xyz) / dot(camera.right.xyz, camera.right.xyz);
    let y = dot(from_eye, camera.up.xyz) / dot(camera.up.xyz, camera.up.xyz);
    var corner_out: Corner;
    corner_out.clip = vec4<f32>(x, y, near.reach.z, ahead);
    corner_out.from_eye = from_eye;
    corner_out.what = vec2<u32>(facing, face.y >> 10u);
    corner_out.ahead = ahead;
    return corner_out;
}

fn looks(facing: u32) -> vec3<f32> {
    switch facing {
        case 0u: { return vec3<f32>(1.0, 0.0, 0.0); }
        case 1u: { return vec3<f32>(-1.0, 0.0, 0.0); }
        case 2u: { return vec3<f32>(0.0, 1.0, 0.0); }
        case 3u: { return vec3<f32>(0.0, -1.0, 0.0); }
        case 4u: { return vec3<f32>(0.0, 0.0, 1.0); }
        default: { return vec3<f32>(0.0, 0.0, -1.0); }
    }
}

// A surface's colour lit as the far eye lights a finished ray (`far.wgsl`'s `lit`, `sunlit` and `shaded`): the sun on
// the way it looks, the sky's light, and from the far eye's light store at its place the sun's shadow and how much
// sky it sees.
fn near_lit(colour: vec3<f32>, light: f32, from_eye: vec3<f32>) -> vec3<f32> {
    var whole = lit(colour, light);
    let sunny = sunlit(colour, light);
    let skyish = whole - sunny;
    var sun = 1.0;
    var sky_open = 1.0;
    if (camera.view.w > 0.5) {
        let store = cell_light(camera.position.xyz + from_eye, f32(1 << u32(FIRST)));
        sun = select(1.0, store.x, camera.follow.y > 0.5);
        sky_open = store.y;
    }
    whole -= sunny * (1.0 - sun);
    let sky = select(1.0, mix(1.0, sky_open, camera.sky.y), camera.sky.x > 0.5);
    let tint = mix(vec3<f32>(1.0), SKY_TINT, camera.sky.z);
    return whole + skyish * (sky * tint - 1.0);
}

// The x-fader [David, 2026-11-16: "i think a x-fader with curves could hide it almost completely so blocks smoothly
// fade into rays"]. How much of a near surface shows at a place: all of it before the band, none at the near faces'
// reach, and across the band along the curve: straight at 1; higher, an S that holds each side longer and crosses
// quicker in the middle (half and half at the band's middle whatever the curve). A band of no width is a hard edge.
fn near_shows(from_eye: vec3<f32>) -> f32 {
    let across = clamp(1.0 - (near.reach.x - length(from_eye.xy)) / max(near.reach.y, 0.001), 0.0, 1.0);
    let curve = max(near.part.w, 1.0);
    let rays = pow(across, curve);
    let blocks = pow(1.0 - across, curve);
    return blocks / max(blocks + rays, 1e-9);
}

@fragment
fn near_face(corner: Corner) -> @location(0) vec4<f32> {
    let shows = near_shows(corner.from_eye);
    if (shows <= 0.0) {
        discard;
    }
    let facing = corner.what.x;
    let way = looks(facing);
    let colour = colours[corner.what.y * 2u + select(1u, 0u, facing == 4u)].rgb;
    let light = max(dot(way, camera.sun.xyz), 0.0);
    // (Into the near eye's own picture, whole: the nearest face wins a pixel, whatever order the chunks come in.
    // Kept multiplied by how much of it shows, for the x-fader to lay over the far picture.)
    return vec4<f32>(to_srgb(clamp(near_lit(colour, light, corner.from_eye), vec3<f32>(0.0), vec3<f32>(1.0))) * shows, shows);
}

// The water's top, over the faces already drawn: the far eye's water (its colour, and how fast it dims what's seen
// through it), by how much water the eye looks through to the face behind.
@fragment
fn near_water(corner: Corner) -> @location(0) vec4<f32> {
    let shows = near_shows(corner.from_eye);
    if (shows <= 0.0) {
        discard;
    }
    let behind = textureLoad(t_depth, vec2<i32>(corner.clip.xy), 0);
    // (A face nearer than the water has the greater depth: it hides it.)
    if (corner.clip.z < behind) {
        discard;
    }
    // How far ahead the face behind is, from its depth; nothing behind (the near faces end): deep water.
    let ahead_behind = select(near.reach.z / max(behind, 1e-12), near.reach.w, behind <= 0.0);
    let through = max(ahead_behind - corner.ahead, 0.0) * length(corner.from_eye) / max(corner.ahead, 0.01);
    let dimmed = 1.0 - exp(-camera.water.w * through);
    let covers = dimmed * shows;
    return vec4<f32>(to_srgb(clamp(near_lit(camera.water.rgb, 1.0, corner.from_eye), vec3<f32>(0.0), vec3<f32>(1.0))) * covers, covers);
}

// Lighting: the sums both the near ground (terrain.wesl) and the far terrain (distant.wesl) light the
// world with, written once [2026-10-29, Counsel's seam plan]. Before, each shader had its own copy,
// kept alike by hand, and every mismatch between near and far crept in through the copies. Each
// shader only gathers its own inputs (the near ground from its blocks and the window round you, the
// far terrain from its smooth surface) and lights them here.

// A value stored square-rooted each way around 0.5 (src/render/light.rs), back to -1 to 1.
fn unroot(v: f32) -> f32 {
    let b = v * 2.0 - 1.0;
    return sign(b) * b * b;
}

// Hill shading's light, `raw` (1 = flat ground), eased toward its darkest (0.35) and brightest
// (1.6) instead of stopped dead, more gradually the lower `fade` is (src/render/distant.rs,
// `soft_limit`, the same sums for the tests).
fn soft_limit(raw: f32, fade: f32) -> f32 {
    let d = raw - 1.0;
    let limit = select(1.6 - 1.0, 1.0 - 0.35, d < 0.0);
    // Never quite 0: pow(0, p) isn't defined on every graphics card.
    let x = max(abs(d) / limit, 1e-6);
    let p = clamp(fade, 0.5, 16.0);
    return 1.0 + sign(d) * limit * x / pow(1.0 + pow(x, p), 1.0 / p);
}

// How much a spot `below` blocks under the cones round it is shaded, 0 to 1: about as much as it
// is below, for small amounts, easing toward 1 by `fade` (as `soft_limit` does).
fn eased(below: f32, fade: f32) -> f32 {
    let x = max(below, 1e-6);
    let p = clamp(fade, 0.5, 16.0);
    return select(x / pow(1.0 + pow(x, p), 1.0 / p), 0.0, below <= 0.0);
}

// The lie of the land's sun and shade: ground rising `rise` blocks north and east across one of the
// distant terrain's finest cells, its hill shading `strength`, the sun's direction (game axes) and
// the fade: 1 on flat ground, more facing the sun, less facing away.
fn hill_light(rise: vec2<f32>, strength: f32, sun: vec3<f32>, fade: f32) -> f32 {
    let normal = normalize(vec3<f32>(-rise.x * strength, -rise.y * strength, 1.0));
    return soft_limit(dot(normal, sun) / sun.z, fade);
}

// Land a little lighter the higher it is: `up` (Z) from the sea to the tint's top.
fn height_tint(up: f32, sea: f32, top: f32, strength: f32) -> f32 {
    return 1.0 + strength * (up - sea) / (top - sea);
}

// The light map at a spot (src/render/light.rs): red sun, green sky hidden (square-rooted), blue
// relief (square-rooted each way), alpha land; `strengths` x on, y shadows, z sky, w relief. The sea
// only takes the shadows. 1 = as painted.
fn light_from_map(map: vec4<f32>, strengths: vec4<f32>) -> f32 {
    let sun = 1.0 - strengths.y * (1.0 - map.r);
    let sky = 1.0 - min(strengths.z * map.g * map.g, 0.9);
    let relief = max(1.0 + strengths.w * unroot(map.b), 0.2);
    return sun * mix(1.0, sky * relief, map.a);
}

// How much of the lie of the land's light (`lie`) the steps' sides take: `share` of it.
fn side_lie(lie: f32, share: f32) -> f32 {
    return mix(1.0, lie, share);
}

// The ground as its blocks show it from where you look: the tops and the steps' sides, each as much
// as the camera sees of it [Counsel's seam plan, step 1]. `rise`: blocks up per block across, north
// and east; a side faces back down the slope (ground rising north shows south-facing sides), and the
// camera sees a face as much as it faces the camera, so a slope facing you shows mostly sides and
// one facing away only tops. `toward`: the way to the camera, game axes (x north, y east, z up).
fn as_seen(top: vec3<f32>, north_south: vec3<f32>, east_west: vec3<f32>, rise: vec2<f32>, toward: vec3<f32>) -> vec3<f32> {
    let top_seen = max(toward.z, 0.0);
    let north_seen = abs(rise.x) * max(-sign(rise.x) * toward.x, 0.0);
    let east_seen = abs(rise.y) * max(-sign(rise.y) * toward.y, 0.0);
    let seen = top_seen + north_seen + east_seen;
    if seen <= 1e-5 {
        return top;
    }
    return (top * top_seen + north_south * north_seen + east_west * east_seen) / seen;
}

// The land as the far terrain draws it: its colour as the near ground's blocks show it on average,
// from where you look: the tops (the lie of the land, the height tint, the shade at the feet of the
// steps, more on the side away from the sun) and the steps' sides, each its own colour and
// brightness, mixed by how much the camera sees of each. The far terrain (distant.wesl) and the far
// copies' plain ground (terrain.wesl, `far_ground`) both draw with it, so they match.
//   `slope`: blocks up per block across, north and east; `up`: the height (Z); `toward`: the way to
//   the camera (game axes); `sun`: xyz toward the sun, w hill shading strength; `tint`: x the height
//   tint's strength, y sea level, z the height it counts up to; `finest`: the distance the slope's
//   light is measured over; `fade`: hill shading's fade; `faces`: x tops, y north/south sides, z
//   east/west sides, w how much of the slope's light the sides take; `matched`: how much the far
//   slopes take on the steps' shade; `foot`, `step`: the steps' feet's shade at this steepness, all
//   round and from the sun's side.
fn land(top_colour: vec3<f32>, side_colour: vec3<f32>, slope: vec2<f32>, up: f32, toward: vec3<f32>, sun: vec4<f32>, tint: vec3<f32>, finest: f32, fade: f32, faces: vec4<f32>, matched: f32, foot: f32, step: f32) -> vec3<f32> {
    let lie = hill_light(slope * finest, sun.w, sun.xyz, fade) * height_tint(up, tint.y, tint.z, tint.x);
    let top = top_colour * faces.x * lie;
    let rise = length(slope);
    if matched <= 0.0 || rise <= 1e-4 {
        return top;
    }
    // The step shadows fall where the slope rises toward the sun (the steps' walls face away).
    let toward_sun = select(0.0, clamp(dot(slope / rise, normalize(sun.xy)), 0.0, 1.0), length(sun.xy) > 1e-5);
    let feet = foot * mix(1.0, step, toward_sun);
    let sides = side_lie(lie, faces.w);
    return as_seen(
        top * mix(1.0, feet, min(matched, 1.0)),
        side_colour * faces.y * sides,
        side_colour * faces.z * sides,
        slope * matched,
        toward,
    );
}

// Bevy's axes to the game's: game x north = Bevy -z, y east = Bevy x, z up = Bevy y.
fn to_game(bevy: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(-bevy.z, bevy.x, bevy.y);
}

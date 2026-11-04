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
    return as_seen_banded(top, top, north_south, east_west, rise, toward, 0.0, -1.0);
}

// The same, the steps' sides drawn where they are, in bands at each whole block of `height` (contour
// bands, round 2: David, "contour bands are how we see shape and depth in land, we just need them to
// all look the same"): each band's share of a block of height is the sides' share of what the camera
// sees, so a pixel `width` (blocks of height across it) that spans several steps comes to the plain
// mix, and a band thinner than a pixel is a faint line, never hit or missed whole. `width` below 0: the
// plain mix (as before).
// `top` is the tops with the shade at the steps' feet spread over them, as the plain mix has it, and
// `open` without it. In bands that shade goes in the bands, at the steps, where the meshes show it, the
// tops between them as `open` (the same mix on average): spread over the tops, it left them darker than
// the bands, the lines lighter than the ground, the steps' shading turned inside out [David, build 44:
// "it's almost like it just inverts the shading between side and top"].
fn as_seen_banded(top: vec3<f32>, open: vec3<f32>, north_south: vec3<f32>, east_west: vec3<f32>, rise: vec2<f32>, toward: vec3<f32>, height: f32, width: f32) -> vec3<f32> {
    let top_seen = max(toward.z, 0.0);
    let north_seen = abs(rise.x) * max(-sign(rise.x) * toward.x, 0.0);
    let east_seen = abs(rise.y) * max(-sign(rise.y) * toward.y, 0.0);
    let sides_seen = north_seen + east_seen;
    let seen = top_seen + sides_seen;
    if seen <= 1e-5 || sides_seen <= 1e-5 {
        return top;
    }
    if width < 0.0 {
        return (top * top_seen + north_south * north_seen + east_west * east_seen) / seen;
    }
    let sides = (north_south * north_seen + east_west * east_seen) / sides_seen;
    let share = sides_seen / seen;
    // The plain mix, kept: the tops' share of the feet's shade moved into the bands, as much of it as
    // leaves a band a quarter of the sides' own at the darkest; any left stays on the tops.
    let plain = mix(top, sides, share);
    let band = max(sides - (open - top) * (1.0 - share) / share, sides * 0.25);
    let between = select(open, (plain - band * share) / (1.0 - share), share < 0.999);
    return mix(between, band, bands_at(height, width, share));
}

// How much of a pixel the bands cover: bands `share` of a block tall, one centred on each whole block
// of `height`, box-filtered over the pixel's `width` (blocks of height).
fn bands_at(height: f32, width: f32, share: f32) -> f32 {
    let x = height + share * 0.5;
    if width < 1e-4 {
        return select(0.0, 1.0, fract(x) < share);
    }
    return (band_sum(x + width * 0.5, share) - band_sum(x - width * 0.5, share)) / width;
}

// How much of the height up to `x` the bands cover (`share` of each block, at its bottom).
fn band_sum(x: f32, share: f32) -> f32 {
    return floor(x) * share + min(fract(x), share);
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
//   round and from the sun's side; `bands`: x the height the contour bands go by, y the pixel's width
//   in it (below 0: no bands, the plain mix; `as_seen_banded`).
fn land(top_colour: vec3<f32>, side_colour: vec3<f32>, slope: vec2<f32>, up: f32, toward: vec3<f32>, sun: vec4<f32>, tint: vec3<f32>, finest: f32, fade: f32, faces: vec4<f32>, matched: f32, foot: f32, step: f32, bands: vec2<f32>) -> vec3<f32> {
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
    return as_seen_banded(
        top * mix(1.0, feet, min(matched, 1.0)),
        top,
        side_colour * faces.y * sides,
        side_colour * faces.z * sides,
        slope * matched,
        toward,
        bands.x,
        bands.y,
    );
}

// Bevy's axes to the game's: game x north = Bevy -z, y east = Bevy x, z up = Bevy y.
fn to_game(bevy: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(-bevy.z, bevy.x, bevy.y);
}

// ── The near world's blocks, as the meshes (terrain.wesl) and the march's whole copies (march.wesl)
// both draw them [one shading recipe, 2026-11-03: the march had its own, and showed where the two
// met: David, "which should i be using right now"]. Each shader gathers its own inputs (a mesh's
// from its corners, the march's from the cell its ray met) and draws them here.

struct TerrainShaderSettings {
    // x top, y bottom, z north/south, w east/west
    face_shade: vec4<f32>,
    fog_color: vec4<f32>,
    // x fog start, y fog end, z fog on, w debug view (1 rectangles, 2 face directions, 3 who drew this)
    fog_and_debug: vec4<f32>,
    // x random texture rotation on, y light only (plain grey), z a far copy's material (far_copies.rs)
    options: vec4<f32>,
    // Survey view: x on, y every how many levels, z darkness, w sea level
    survey: vec4<f32>,
    // Underground (src/render/sky_light.rs): x daylight on, y the test lantern's reach (blocks; 0 off),
    // z its brightness, w how bright the sky is (0 to 1: a stand-in for night)
    underground: vec4<f32>,
    // Placed light (src/render/placed_light.rs, in the vertex): x on, y how much it flickers, z how fast
    placed: vec4<f32>,
}

// How the ground is lit (src/render/light.rs).
struct TerrainLight {
    // The world light map: x on, y shadows, z sky, w relief (strengths)
    light: vec4<f32>,
    // x the map's half width, y the light map's width (blocks)
    light_area: vec4<f32>,
    // xyz toward the sun (game axes), w hill shading strength
    sun: vec4<f32>,
    // The window of the ground around you: x north and y east of its north-west corner, z its side
    // (blocks), w on
    near_area: vec4<f32>,
    // x height tint, y the distance slopes are measured over (blocks), z sea level, w the highest a
    // height can be
    hill: vec4<f32>,
    // x how hill shading fades toward its limits (see distant.rs, `soft_limit`), y foot shading, z
    // step shadows (strengths), w the height the height tint counts up to (as the distant tiles)
    shade: vec4<f32>,
    // x how much of the lie of the land's sun and shade the steps' sides take (0 to 1)
    sides: vec4<f32>,
    // For the far copies' plain ground, lit as the far terrain is (`far_ground`): x how much the far
    // slopes take on the steps' shade, y the steepest slope the tables reach; and the steps' feet's
    // shade by steepness (from flat to `far.y` in 32 steps, four to a vec4), all round and from the
    // sun's side (the far terrain's own tables).
    far: vec4<f32>,
    foot: array<vec4<f32>, 9>,
    step: array<vec4<f32>, 9>,
    // The floating origin (src/coords.rs): x north, y east (blocks), what drawn positions are
    // measured from
    origin: vec4<f32>,
}

// Texture coordinates come straight from the position, so textures tile once per block.
// Side faces: image top = up (+Z). Top/bottom faces: image top = north (+X).
// "Right" on the image is chosen so nothing appears mirrored when seen from outside.
fn face_uv(face: u32, p: vec3<f32>) -> vec2<f32> {
    switch face {
        case 0u: { return vec2<f32>(-p.y, -p.z); }  // north face: seen from the north, east is on the left
        case 1u: { return vec2<f32>(p.y, -p.z); }   // south face
        case 2u: { return vec2<f32>(p.x, -p.z); }   // east face
        case 3u: { return vec2<f32>(-p.x, -p.z); }  // west face
        case 4u: { return vec2<f32>(p.y, -p.x); }   // top
        default: { return vec2<f32>(-p.y, -p.x); }  // bottom
    }
}

// Which way each face points (game axes).
fn face_normal(face: u32) -> vec3<f32> {
    switch face {
        case 0u: { return vec3<f32>(1.0, 0.0, 0.0); }
        case 1u: { return vec3<f32>(-1.0, 0.0, 0.0); }
        case 2u: { return vec3<f32>(0.0, 1.0, 0.0); }
        case 3u: { return vec3<f32>(0.0, -1.0, 0.0); }
        case 4u: { return vec3<f32>(0.0, 0.0, 1.0); }
        default: { return vec3<f32>(0.0, 0.0, -1.0); }
    }
}

// Scramble a block position into a "random" number. The same block always gives the same number.
fn block_hash(p: vec3<i32>, face: u32) -> u32 {
    var h = (bitcast<u32>(p.x) * 73856093u) ^ (bitcast<u32>(p.y) * 19349663u)
        ^ (bitcast<u32>(p.z) * 83492791u) ^ (face * 2654435761u);
    h = h ^ (h >> 16u);
    h = h * 0x7feb352du;
    h = h ^ (h >> 15u);
    h = h * 0x846ca68bu;
    h = h ^ (h >> 16u);
    return h;
}

// A block's face turned in quarter turns round the block's centre, by its random number
// (`block_hash`), the same for a block wherever it's drawn.
fn turned(uv: vec2<f32>, block: vec3<i32>, face: u32) -> vec2<f32> {
    let turns = (block_hash(block, face) >> 8u) & 3u;
    let cell = floor(uv);
    var f = uv - cell;
    switch turns {
        case 1u: { f = vec2<f32>(1.0 - f.y, f.x); }
        case 2u: { f = vec2<f32>(1.0 - f.x, 1.0 - f.y); }
        case 3u: { f = vec2<f32>(f.y, 1.0 - f.x); }
        default: {}
    }
    return cell + f;
}

// Classic block shading: brightness by which way the face points (`shades`: x top, y bottom, z
// north/south, w east/west).
fn face_shade(face: u32, shades: vec4<f32>) -> f32 {
    var shade = shades.x;
    if face == 5u {
        shade = shades.y;
    } else if face <= 1u {
        shade = shades.z;
    } else if face <= 3u {
        shade = shades.w;
    }
    return shade;
}

// How lit a spot on the ground is (1 = as painted), from what the shader read for it:
//   - `map_light`, the world light map there (shadows, valley darkening, relief; `light_from_map`),
//     on every face;
//   - `near`, the window of the ground round you there (its alpha 0 if it's off, or the spot is off
//     it): the lie of the land's sun and shade, from the slope of the ground under them (a block
//     world's tops are all flat, so which way a face points says nothing about it), and the height
//     tint: on tops, and as much as set on the steps' sides;
//   - `cones`, the cone map there (`cones_at`, read for tops only): the feet of the steps, on tops.
// Faces well below the ground (caves, overhangs) keep their plain light, for now. `up` is the spot's
// height; `sun`, `hill`, `shade` and `sides` are `TerrainLight`'s.
fn ground_lit(map_light: f32, near: vec4<f32>, cones: vec2<f32>, face: u32, up: f32, sun: vec4<f32>, hill: vec4<f32>, shade: vec4<f32>, sides: vec4<f32>) -> f32 {
    var light = map_light;
    if near.a > 0.5 {
        let sea = hill.z;
        let surface = sea + near.b * (hill.w - sea);
        if face != 5u {
            // The rise across one distant cell, as the distant terrain measures it.
            let rise = vec2<f32>(unroot(near.r), unroot(near.g)) * 4.0 * hill.y;
            let lie = hill_light(rise, sun.w, sun.xyz, shade.x) * height_tint(up, sea, shade.w, hill.x);
            var lit = side_lie(lie, sides.x);
            if face == 4u {
                // The feet of steps: how far this top is below the cones of the ground round it
                // (0 on a top, a rim or open ground), eased like the hill shading.
                let fade = shade.x;
                let feet = (1.0 - shade.y * eased(cones.x - up, fade)) * (1.0 - shade.z * eased(cones.y - up, fade));
                lit = lie * feet;
            }
            // The sea's surface stays flat, as on the distant terrain.
            let water = surface <= sea + 0.5 && up <= sea + 0.5;
            light = light * select(lit, 1.0, water);
        }
        // Well below the ground: plain light.
        light = mix(1.0, light, smoothstep(surface - 24.0, surface - 12.0, up));
    }
    return light;
}

// How lit a face is underground (src/render/sky_light.rs; docs/caves.md, stop 3): its daylight, eased
// (squared, so it falls away quickly into the dark), or the test lantern round the camera
// (`distance` from it), whichever is brighter; and the placed light on top, in its colour
// (src/render/placed_light.rs; docs/objects.md, stop 3), eased the same way and wavering a little with
// `time`, as a fire's does. Pitch black where none reaches [David: "pitch black unless something is
// making light down there"]. `underground` and `placed_light` are `TerrainShaderSettings`'.
fn underground_lit(daylight: f32, placed: vec3<f32>, distance: f32, underground: vec4<f32>, placed_light: vec4<f32>, time: f32) -> vec3<f32> {
    var lit = 1.0;
    if underground.x > 0.5 {
        lit = daylight * daylight;
    }
    // How bright the sky is: all daylight dimmed by it (a stand-in for night, David 2026-11-02).
    lit = lit * underground.w;
    let reach = underground.y;
    if reach > 0.0 {
        let near = 1.0 - clamp(distance / reach, 0.0, 1.0);
        lit = max(lit, underground.z * near * near);
    }
    var light = vec3<f32>(lit);
    if placed_light.x > 0.5 {
        // The flicker: two slow waves and a quicker one, so it never quite repeats.
        let t = time * placed_light.z;
        let waver = 0.5 + 0.25 * sin(t * 6.1) + 0.15 * sin(t * 13.7 + 1.3) + 0.1 * sin(t * 29.3 + 2.9);
        let flicker = 1.0 - placed_light.y * waver;
        light = light + placed * placed * flicker;
    }
    return min(light, vec3<f32>(1.0));
}

// ── The far eye's layers marched by one dispatch (the Oracle's step 2, its one-dispatch form) ──
//
// Not a shader on its own: `far_eye.rs` puts this in `far.wgsl`'s place of one layer's outputs (between its two
// marks), and every use of the camera there becomes a read of this invocation's layer's row of `cameras` (in place: a
// copy of the whole row for each invocation cost, build 155). Every invocation works out which layer's pixel it is
// and marches it with `far.wgsl`'s own `march_pixel`. A finer layer's lead (where it's wanted, where its rays start)
// is the coarser layer's of the frame before.

// A picture for each layer (0 the base), and all their distances in one texture: the base, the middle layer and the
// finest side by side from its left (four written textures in a pass, the library's default most).
@group(0) @binding(5) var picture0: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(17) var picture1: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(19) var picture2: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(6) var how_far: texture_storage_2d<rgba32float, write>;
// The frame before's words, the base's then the middle layer's in one run; and this frame's, the same way.
@group(0) @binding(8) var<storage, read> worth_in: array<u32>;
@group(0) @binding(9) var<storage, read_write> worth_out: array<u32>;
// The frame before's distances, laid out the same.
@group(0) @binding(10) var far_prev: texture_2d<f32>;

// The cut's counts: how many rays this frame's list holds, and the frame before's; then how many lengths this
// frame's list of lengths holds, and the frame before's. Words written by many rays at once, so only ever touched by
// the few rays that are long, and never read here. [Build 181 kept the tiles' starts in the same buffer, and every
// ray's read of its tile's start became a read and a write locked together, on a word its 63 neighbours were locking
// too: 4 to 11% at every mark.]
@group(0) @binding(7) var<storage, read_write> long_rays: array<atomic<u32>>;
// The cut's own store, plain words, each written by one thread (`far_eye.rs` has the same layout):
// - the two lists of long rays, three words a place: the ray's pixel and its layer; its cuts (a bit for each of the
//   sixteen places a ray may be cut at, `far.wgsl`'s `cut_place`: set, it's cut there); where its lengths start in
//   the list of lengths;
// - for each place in each list, how its ray stood at each of the sixteen places (how much it still let through;
//   under nothing if it was in haze there, or it isn't known), for the frame after to begin its lengths by;
// - the two lists of lengths: for each length beyond a ray's first, its ray's place in the list of rays. A ray's
//   lengths are side by side in it, and the strip's threads are its places, so the strip is packed [build 183 gave
//   every cut ray sixteen threads whatever its length, and four fifths of the strip's thread-steps were waiting:
//   builds 183 and 184's flights];
// - the stacks, `STACK` words each: one for each place in the list of rays (its ray's first length, marched by the
//   pixel's own thread), then one for each place in the list of lengths. What the length gathered (3) and the sun's
//   part of that (3); how much it ended letting through; where it last gathered with over half still let through
//   (its own first surface, if it ended letting through a half or less); where it first touched anything; its steps
//   with the level read where it last gathered, and in the top bit whether it ended in air; what it began letting
//   through; where it began; where it stopped.
@group(0) @binding(25) var<storage, read_write> cut: array<u32>;
const STACK: u32 = 13u;

// Where each tile's rays start, by the one dispatch: under the base layer's part of the frame before's distances
// (nothing else is there: the base's rows are fewer than the finest's). The start pass is given that picture to
// write; the march reads it as it reads the rest.
fn put_start(tile: vec2<u32>, across: u32, v: f32) {
    textureStore(how_far, vec2<i32>(tile) + vec2<i32>(0, i32(cameras[0].view.y)), vec4<f32>(v, 0.0, 0.0, 0.0));
}

fn tile_start(tile: vec2<u32>, across: u32) -> f32 {
    return textureLoad(far_prev, vec2<i32>(tile) + vec2<i32>(0, i32(cameras[0].view.y)), 0).x;
}

// Which layer this invocation marches (0 the base).
var<private> layer_now: u32;

// Where the middle layer's words start in the runs: after the base's.
fn base_pixels() -> u32 {
    return u32(cameras[0].view.x) * u32(cameras[0].view.y);
}

fn put_picture(at: vec2<i32>, v: vec4<f32>) {
    switch layer_now {
        case 0u { textureStore(picture0, at, v); }
        case 1u { textureStore(picture1, at, v); }
        default { textureStore(picture2, at, v); }
    }
}

// Where a layer's distances start in their texture: after the layers before it.
fn far_left(layer: u32) -> i32 {
    var left = 0;
    if (layer > 0u) {
        left += i32(cameras[0].view.x);
    }
    if (layer > 1u) {
        left += i32(cameras[1].view.x);
    }
    return left;
}

fn put_far(at: vec2<i32>, v: vec4<f32>) {
    textureStore(how_far, at + vec2<i32>(far_left(layer_now), 0), v);
}

// This layer's word for the layer below (the finest has none below and never writes).
fn put_worth(i: u32, v: u32) {
    worth_out[i + select(0u, base_pixels(), layer_now == 1u)] = v;
}

// The layer above's word (the middle layer reads the base's, the finest the middle layer's).
fn get_worth(i: u32) -> u32 {
    return worth_in[i + select(0u, base_pixels(), layer_now == 2u)];
}

// The layer above's last pixel, and its distance at one (never asked by the base).
fn far_above_edge() -> vec2<i32> {
    return vec2<i32>(cameras[max(layer_now, 1u) - 1u].view.xy) - 1;
}

fn get_far_above(at: vec2<i32>) -> f32 {
    return textureLoad(far_prev, at + vec2<i32>(far_left(max(layer_now, 1u) - 1u), 0), 0).x;
}

// ── The cut: where things are in its store ──

// A place in a list of rays (`which` list): its three words.
fn list_at(which: u32, slot: u32) -> u32 {
    return (which * u32(cameras[0].list.z) + slot) * 3u;
}

// How a listed ray stood at each of the sixteen places (17 numbers, the first unused).
fn befores_at(which: u32, slot: u32) -> u32 {
    let holds = u32(cameras[0].list.z);
    return 6u * holds + (which * holds + slot) * 17u;
}

// A list of lengths (as many places as `cut.y`).
fn pieces_at(which: u32) -> u32 {
    return 40u * u32(cameras[0].list.z) + which * u32(cameras[0].cut.y);
}

// The stack of a listed ray's first length, and of a place in the list of lengths.
fn own_stack(slot: u32) -> u32 {
    return 40u * u32(cameras[0].list.z) + 2u * u32(cameras[0].cut.y) + slot * STACK;
}

fn piece_stack(piece: u32) -> u32 {
    return own_stack(u32(cameras[0].list.z)) + piece * STACK;
}

// ── The cut: the lists ──

// The place in this frame's list a ray was given by `list_long`, for what's written after it (`NOT_CUT`: none).
var<private> listed_now: u32;

// A long ray listed: a ray that took `list.y` steps or more adds its pixel to this frame's list, and its place in
// the list, one up, goes beside its distance for the frame after (0: not long, or the list was full). A ray joined
// from its lengths stays listed down to half of that: listed with no cuts it costs the strip nothing, and it
// doesn't go in and out of the list frame by frame. [Rays long enough to set the far eye's time are a few hundred,
// in a thin band at the horizon, and mostly the same from frame to frame: David, 2026-11-14, "we have to reduce the
// longest rays, one by one in order of length".]
fn list_long(id: vec2<u32>, steps: f32, joined: bool) -> f32 {
    let long = cameras[0].list.y;
    if (steps < select(long, long * 0.5, joined)) {
        return 0.0;
    }
    let now = u32(cameras[0].list.x);
    let slot = atomicAdd(&long_rays[now], 1u);
    if (slot >= u32(cameras[0].list.z)) {
        return 0.0;
    }
    // (No cuts until `give_cuts` says: its join's, or `cut_begin`'s for a ray listed whole.)
    let at = list_at(now, slot);
    cut[at] = (layer_now << 30u) | (id.y << 15u) | id.x;
    cut[at + 1u] = 0u;
    cut[at + 2u] = 0u;
    listed_now = slot;
    return f32(slot + 1u);
}

// A ray in this frame's list given its cuts for the frame after, and its lengths their places in the list of
// lengths, side by side. If the list of lengths is full, it's left uncut: marched whole.
fn give_cuts(slot: u32, asked: u32) {
    let now = u32(cameras[0].list.x);
    var cuts = asked & 0xFFFFu;
    var first = 0u;
    let lengths = countOneBits(cuts);
    if (lengths > 0u) {
        first = atomicAdd(&long_rays[2u + now], lengths);
        if (first + lengths > u32(cameras[0].cut.y)) {
            cuts = 0u;
            first = 0u;
        }
    }
    let at = list_at(now, slot);
    cut[at + 1u] = cuts;
    cut[at + 2u] = first;
    let pieces = pieces_at(now) + first;
    for (var i = 0u; i < countOneBits(cuts); i++) {
        cut[pieces + i] = slot;
    }
}

// The place nearest `distance` along a pixel's ray that it may be cut at (1 to 16; outside that: none there).
fn place_near(distance: f32, shift: f32) -> i32 {
    return i32(round(2.0 * log2(max(distance, 1.0) / cameras[0].cut.x) - shift)) + 1;
}

// A ray just listed, marched whole this frame: its first cuts. As many lengths as its steps over 80 say, the cuts
// spread evenly (by doublings of distance, as a ray's steps fall) over the stretch it marched. How it stands at
// each isn't known: the frame after, each length takes up letting through all, and looks before it skips.
fn cut_begin(id: vec2<u32>, starts: f32, seen: Seen) {
    if (listed_now == NOT_CUT || cameras[0].cut.x <= 0.0) {
        return;
    }
    let befores = befores_at(u32(cameras[0].list.x), listed_now);
    for (var k = 0u; k < 17u; k++) {
        cut[befores + k] = bitcast<u32>(-1.0);
    }
    let shift = scatter(id * 2u + vec2<u32>(1u, 1u));
    let low = max(starts, cameras[0].cut.x * 0.7);
    let high = max(seen.ended_at, low);
    let lengths = min(u32(ceil(seen.steps / 80.0)), 17u);
    var cuts = 0u;
    for (var j = 1u; j < lengths; j++) {
        let k = place_near(low * pow(high / low, f32(j) / f32(lengths)), shift);
        if (k >= 1 && k <= 16) {
            let at = cut_place(u32(k), shift);
            if (at > low && at < high) {
                cuts |= 1u << u32(k - 1);
            }
        }
    }
    give_cuts(listed_now, cuts);
}

// The ray at a place in the frame before's list: its pixel (x, y) and its layer; layer 3 if no ray was listed there
// the frame before (the ray's own word beside its distance says where it was listed, and is asked: the list's word
// may be an older frame's).
fn listed_pixel(slot: u32) -> vec3<u32> {
    if (slot >= u32(cameras[0].list.z)) {
        return vec3<u32>(0u, 0u, 3u);
    }
    let entry = cut[list_at(1u - u32(cameras[0].list.x), slot)];
    let layer = entry >> 30u;
    let pixel = vec2<u32>(entry & 0x7FFFu, (entry >> 15u) & 0x7FFFu);
    if (layer == 3u || any(pixel >= vec2<u32>(cameras[layer].view.xy))) {
        return vec3<u32>(0u, 0u, 3u);
    }
    let listed = textureLoad(far_prev, vec2<i32>(pixel) + vec2<i32>(far_left(layer), 0), 0).w;
    if (u32(listed) != slot + 1u) {
        return vec3<u32>(0u, 0u, 3u);
    }
    return vec3<u32>(pixel, layer);
}

// A pixel's place in the frame before's list, if its ray was listed (it's marched as its lengths this frame): from
// beside its distance.
fn cut_slot(id: vec2<u32>) -> u32 {
    if (cameras[0].cut.x <= 0.0) {
        return NOT_CUT;
    }
    let listed = textureLoad(far_prev, vec2<i32>(id) + vec2<i32>(far_left(layer_now), 0), 0).w;
    return select(NOT_CUT, u32(listed) - 1u, listed > 0.5);
}

// The first place a ray in the frame before's list is cut at (17: it isn't cut, its own thread marches all of it).
fn cut_first(slot: u32) -> u32 {
    let cuts = cut[list_at(1u - u32(cameras[0].list.x), slot) + 1u];
    return select(firstTrailingBit(cuts) + 1u, 17u, cuts == 0u);
}

// How a ray in the frame before's list stood, that frame, at the place `place`: how much it still let through; under
// nothing if it was in haze, or it isn't known.
fn cut_before(slot: u32, place: u32) -> f32 {
    return bitcast<f32>(cut[befores_at(1u - u32(cameras[0].list.x), slot) + place]);
}

// What a length gathered, put by for the join; what it began letting through; where it began and where it stopped.
fn put_stack(at: u32, seen: Seen, began: f32, starts: f32) {
    let gathered = seen.skyish + seen.sunny;
    cut[at] = bitcast<u32>(gathered.r);
    cut[at + 1u] = bitcast<u32>(gathered.g);
    cut[at + 2u] = bitcast<u32>(gathered.b);
    cut[at + 3u] = bitcast<u32>(seen.sunny.r);
    cut[at + 4u] = bitcast<u32>(seen.sunny.g);
    cut[at + 5u] = bitcast<u32>(seen.sunny.b);
    cut[at + 6u] = bitcast<u32>(seen.sky_share);
    cut[at + 7u] = bitcast<u32>(seen.reached);
    cut[at + 8u] = bitcast<u32>(seen.touched);
    cut[at + 9u] = (u32(seen.steps) << 5u) | u32(seen.reached_level + 1) | (u32(seen.ended_in_air) << 31u);
    cut[at + 10u] = bitcast<u32>(began);
    cut[at + 11u] = bitcast<u32>(starts);
    cut[at + 12u] = bitcast<u32>(seen.ended_at);
}

// ── The cut: the join ──

// For each ray in the frame before's list, its lengths' stacks joined front to back (each lets through to the next
// what it didn't cover), into what the whole ray would have seen: its first surface is where what's let through
// first falls to a half, as the march has it. Then the ray is finished into its pixel as any ray is: lit once, by
// that surface. And the ray is cut again for the frame after, by what its lengths took: a length of over 120 steps
// gets a cut put back near the middle of the stretch it marched; two neighbours of under 80 steps together lose
// the cut between them; and where the ray is covered, the cuts beyond go, so the length that reaches the cover runs
// on to the ray's end and stops there as a whole ray does (nothing marches behind a ray's cover). A pass of its own
// after the march, 64 x 64 threads, one a place in the list.
@compute @workgroup_size(8, 8)
fn join_pass(@builtin(global_invocation_id) id: vec3<u32>) {
    if (cameras[0].cut.x <= 0.0) {
        return;
    }
    let listed = id.y * 64u + id.x;
    let pixel = listed_pixel(listed);
    if (pixel.z == 3u) {
        return;
    }
    layer_now = pixel.z;
    let at = pixel.xy;
    // Shown as much as the layer above says, as its own thread found; not at all, and its threads put nothing by.
    var shown = 1.0;
    if (camera.layer.x > 0.0) {
        let parent = parent_of(at);
        let worth = get_worth(parent.y * u32(camera.layer.x) + parent.x) & 0xFFu;
        if (worth == 0u) {
            return;
        }
        shown = f32(worth) / 255.0;
    }
    let d = ray_through(at);
    let shift = scatter(at * 2u + vec2<u32>(1u, 1u));
    let before = list_at(1u - u32(cameras[0].list.x), listed);
    let cuts = cut[before + 1u];
    let first_piece = cut[before + 2u];
    let lengths = countOneBits(cuts);
    var gathered = vec3<f32>(0.0);
    var sunny = vec3<f32>(0.0);
    var through = 1.0;
    var first = camera.reach.x;
    var first_level = -1;
    var touched = camera.reach.x;
    var steps = 0.0;
    var chain = 0.0;
    // How the ray stands at each place, for the frame after; and its cuts for the frame after.
    var befores: array<f32, 17>;
    for (var k = 0u; k < 17u; k++) {
        befores[k] = -1.0;
    }
    var next = cuts;
    var rest = cuts;
    var from_cut = 0u;
    var in_air = true;
    // The length before's steps, if it may still lose the cut after it (not one just cut in two, or just joined).
    var steps_before = -1.0;
    for (var i = 0u; i <= lengths; i++) {
        let to_cut = select(firstTrailingBit(rest) + 1u, 17u, rest == 0u);
        rest &= rest - 1u;
        if (from_cut > 0u) {
            befores[from_cut] = select(-through, through, in_air);
        }
        if (through < 0.01) {
            // Covered before this length: its cut and those beyond go.
            next &= (1u << (from_cut - 1u)) - 1u;
            break;
        }
        var s = own_stack(listed);
        if (i > 0u) {
            s = piece_stack(first_piece + i - 1u);
        }
        let word = cut[s + 9u];
        let took = f32((word >> 5u) & 0x3FFFFFFu);
        steps += took;
        chain = max(chain, took);
        // What the length began letting through, and so how its numbers stand to the ray's as it is this frame:
        // scaled by `now`.
        let began = clamp(bitcast<f32>(cut[s + 10u]), 0.05, 1.0);
        let now = through / began;
        let ended = bitcast<f32>(cut[s + 6u]);
        let reached = bitcast<f32>(cut[s + 7u]);
        let touch = bitcast<f32>(cut[s + 8u]);
        touched = min(touched, touch);
        gathered += now * vec3<f32>(bitcast<f32>(cut[s]), bitcast<f32>(cut[s + 1u]), bitcast<f32>(cut[s + 2u]));
        sunny += now * vec3<f32>(bitcast<f32>(cut[s + 3u]), bitcast<f32>(cut[s + 4u]), bitcast<f32>(cut[s + 5u]));
        let after = now * ended;
        if (through > 0.5 && after <= 0.5 && touch < camera.reach.x) {
            // The ray's first surface is in this length: where the ray has let through a half. If the length began
            // as the ray stands this frame, that's where the length itself last gathered with over a half let
            // through (`reached`), exactly. If the ray stands otherwise now (the view moved, or the ray is new to
            // the list): between the length's first touch and there, as far along as a surface thinning evenly
            // would put it. [Build 183's first pictures: a ray's first surface a tenth out, and the layer below was
            // drawn a quarter more or less over a skyline: it fades in by that distance.]
            if (reached < camera.reach.x) {
                let at_half = 0.5 / through;
                let at_reached = clamp(max(ended, 0.5) / began, 1e-4, 0.9999);
                first = mix(touch, reached, clamp(log(at_half) / log(at_reached), 0.0, 1.0));
                first_level = i32(word & 31u) - 1;
            } else {
                first = touch;
                first_level = level_at(touch);
            }
        }
        // The places inside this length that aren't cuts: how the ray stands there is taken evenly between the
        // length's ends, and looked at first (it's only asked if one is made a cut).
        for (var k = from_cut + 1u; k < min(to_cut, 17u); k++) {
            let along = f32(k - from_cut) / f32(to_cut - from_cut);
            befores[k] = -max(through * pow(clamp(after / through, 1e-4, 1.0), along), 1e-4);
        }
        // Cut again for the frame after.
        if (took > 120.0) {
            // A cut put back at the place nearest the middle of the stretch it marched (by doublings), if a place
            // lies inside it.
            let low = max(bitcast<f32>(cut[s + 11u]), cameras[0].cut.x * 0.7);
            let high = max(bitcast<f32>(cut[s + 12u]), low);
            let k = clamp(place_near(sqrt(low * high), shift), i32(from_cut) + 1, i32(min(to_cut, 17u)) - 1);
            if (k > i32(from_cut) && k < i32(to_cut) && k <= 16) {
                next |= 1u << u32(k - 1);
            }
            steps_before = -1.0;
        } else if (steps_before >= 0.0 && steps_before + took < 80.0) {
            // It and the length before are under 80 steps together: the cut between them goes.
            next &= ~(1u << (from_cut - 1u));
            steps_before = -1.0;
        } else {
            steps_before = took;
        }
        through = after;
        in_air = (word >> 31u) == 1u;
        from_cut = to_cut;
    }
    var seen: Seen;
    seen.colour = gathered + through * sky(d);
    seen.distance = first;
    seen.steps = steps;
    seen.level = first_level;
    seen.sunny = sunny;
    seen.at = camera.position.xyz + d * first;
    seen.wide = shadow_wide(first, first_level);
    seen.skyish = gathered - sunny;
    seen.touched = touched;
    seen.sky_share = through;
    seen.reached = first;
    seen.reached_level = first_level;
    listed_now = NOT_CUT;
    finish(at, d, shown, seen, true, chain);
    // Still in the list: how it stood this frame, and its cuts, for the frame after.
    if (listed_now != NOT_CUT) {
        let stood = befores_at(u32(cameras[0].list.x), listed_now);
        for (var k = 0u; k < 17u; k++) {
            cut[stood + k] = bitcast<u32>(befores[k]);
        }
        give_cuts(listed_now, next);
    }
}

// The dispatch's grid: the base, the middle layer and the finest side by side across it, each from its top row (each
// rounded up to whole groups of 8), so the base's rows are the middle layer's too and neither goes first [the grid's
// order matters: builds 155 and 156]. `march_pixel` turns away what falls outside its layer.
// Past the layers, a strip of threads for the cut rays' lengths beyond their first: a thread a place in the frame
// before's list of lengths, row after row from the strip's top.
@compute @workgroup_size(8, 8)
fn march_all(@builtin(global_invocation_id) id: vec3<u32>) {
    let base_across = (u32(cameras[0].view.x) + 7u) / 8u * 8u;
    let middle_across = (u32(cameras[1].view.x) + 7u) / 8u * 8u;
    let fine_across = (u32(cameras[2].view.x) + 7u) / 8u * 8u;
    listed_now = NOT_CUT;
    // (One call of the march at the end, whichever thread this is: called from two places, the whole march would be
    // in the program twice.)
    var at = id.xy;
    var given = NOT_CUT;
    var from_cut = 0u;
    var to_cut = 17u;
    var stack = 0u;
    if (id.x < base_across) {
        layer_now = 0u;
    } else if (id.x < base_across + middle_across) {
        at.x -= base_across;
        layer_now = 1u;
    } else if (id.x < base_across + middle_across + fine_across) {
        at.x -= base_across + middle_across;
        layer_now = 2u;
    } else {
        if (cameras[0].cut.x <= 0.0) {
            return;
        }
        let piece = id.y * u32(cameras[0].list.w) + (id.x - (base_across + middle_across + fine_across));
        if (piece >= u32(cameras[0].cut.y)) {
            return;
        }
        let before = 1u - u32(cameras[0].list.x);
        let slot = cut[pieces_at(before) + piece];
        let pixel = listed_pixel(slot);
        if (pixel.z == 3u) {
            return;
        }
        // Its ray's cuts say whether this place is one of its lengths' (the list's word may be an older frame's),
        // and which: the cut it begins at and the cut it stops at (17: it runs to the ray's end).
        let cuts = cut[list_at(before, slot) + 1u];
        let first = cut[list_at(before, slot) + 2u];
        if (piece < first || piece - first >= countOneBits(cuts)) {
            return;
        }
        var rest = cuts;
        for (var i = first; i < piece; i++) {
            rest &= rest - 1u;
        }
        from_cut = firstTrailingBit(rest) + 1u;
        rest &= rest - 1u;
        to_cut = select(firstTrailingBit(rest) + 1u, 17u, rest == 0u);
        layer_now = pixel.z;
        at = pixel.xy;
        given = slot;
        stack = piece_stack(piece);
    }
    march_pixel(at, given, from_cut, to_cut, stack);
}

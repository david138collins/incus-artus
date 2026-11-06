// ── Sight: a cell's sums on the card (the regroup, phase 3.1b; `sight/cell.rs` is the reference) ──
//
// Three passes over a region of columns (`sight_region`: its least corner's x and y, how many columns across
// and along). The grounds (the region and three columns round it, for the slope mineralogy reads), then
// what mineralogy chose (the region and one round it, for the faces at the cells' edges), then any cells
// whose columns lie inside the region. Each pass's answers are the next one's inputs.

@group(0) @binding(1) var<storage, read> sight_region: array<i32>;
// Each column's ground (−1 past the map) and the water over it (0 none).
@group(0) @binding(2) var<storage, read_write> sight_grounds: array<vec2<i32>>;
// Each column's ground, water, and what each distributor chose (`choose_blocks`).
@group(0) @binding(3) var<storage, read_write> sight_chosen: array<vec4<i32>>;
// The cells asked for: least corner x, y, z, and size.
@group(0) @binding(4) var<storage, read> sight_cells: array<vec4<i32>>;
// Their sums (`Sums`, 17 numbers each): solid, water, the faces by direction (6), colour (3), light,
// light squared (two halves), height, height squared (two halves).
@group(0) @binding(5) var<storage, read_write> sight_sums: array<u32>;

const SIGHT_SUMS: u32 = 17u;

@compute @workgroup_size(64)
fn sight_grounds_pass(@builtin(global_invocation_id) id: vec3<u32>) {
    let across = sight_region[2] + 6;
    let i = i32(id.x);
    if (i >= across * (sight_region[3] + 6)) {
        return;
    }
    let x = sight_region[0] - 3 + i % across;
    let y = sight_region[1] - 3 + i / across;
    let column = world_column(x * 256 + 128, y * 256 + 128);
    if (column.z == 0) {
        sight_grounds[i] = vec2<i32>(-1, 0);
    } else {
        sight_grounds[i] = column.xy;
    }
}

fn sight_ground(x: i32, y: i32) -> vec2<i32> {
    return sight_grounds[(y - sight_region[1] + 3) * (sight_region[2] + 6) + (x - sight_region[0] + 3)];
}

// A neighbour's ground for the slope; its own past the map (as `CompiledMap::chunk` measures it).
fn sight_ground_or(x: i32, y: i32, own: i32) -> i32 {
    let g = sight_ground(x, y).x;
    if (g < 0) {
        return own;
    }
    return g;
}

@compute @workgroup_size(64)
fn sight_choices_pass(@builtin(global_invocation_id) id: vec3<u32>) {
    let across = sight_region[2] + 2;
    let i = i32(id.x);
    if (i >= across * (sight_region[3] + 2)) {
        return;
    }
    let x = sight_region[0] - 1 + i % across;
    let y = sight_region[1] - 1 + i / across;
    let here = sight_ground(x, y);
    if (here.x < 0) {
        sight_chosen[i] = vec4<i32>(-1, 0, 0, 0);
        return;
    }
    // The slope (`terrain::slope_across`): the rise across four blocks north and east, its length in 16ths
    // of a block by a whole square root, over four.
    var slope = 0;
    if (data[hu(H_MINERALOGY) + 7u] != 0) {
        let north_rise = sight_ground_or(x + 2, y, here.x) - sight_ground_or(x - 2, y, here.x);
        let east_rise = sight_ground_or(x, y + 2, here.x) - sight_ground_or(x, y - 2, here.x);
        let squared = min(u32(north_rise * north_rise + east_rise * east_rise), 0xFFFFFFFFu / 256u);
        slope = i32(wide_sqrt(vec2<u32>(squared * 256u, 0u))) << 10u;
    }
    var col: Col;
    col.north = x * 256 + 128;
    col.east = y * 256 + 128;
    col.ground = here.x << 16u;
    col.water = here.y;
    col.slope = slope;
    let climate = climate_at(col.north, col.east, col.ground);
    col.temperature = 13107;
    col.rain = 39322;
    if (climate.z != 0) {
        col.temperature = climate.x;
        col.rain = climate.y;
    }
    sight_chosen[i] = vec4<i32>(here.x, here.y, choose_blocks(col), 0);
}

fn sight_column(x: i32, y: i32) -> vec4<i32> {
    return sight_chosen[(y - sight_region[1] + 1) * (sight_region[2] + 2) + (x - sight_region[0] + 1)];
}

// What a column is made of (`Mineralogy::whole_layers`): stone, then each distributor's choice, all of it or
// its top only.
struct SightLayers {
    top: i32,
    count: i32,
    depth: i32,
    under: array<vec2<i32>, 4>,
}

fn sight_layers(choices: i32) -> SightLayers {
    let m = hu(H_MINERALOGY);
    let distributors = min(u32(data[m + 1u]), 4u);
    let table = u32(data[m + 2u]);
    var layers: SightLayers;
    layers.top = data[m + 5u];
    layers.count = 0;
    layers.depth = 0;
    for (var d = 0u; d < distributors; d++) {
        let chosen = ((choices >> (8u * d)) & 0xFF) - 1;
        if (chosen < 0) {
            continue;
        }
        let candidate = u32(data[table + d * 7u + 4u]) + u32(chosen) * 16u;
        layers.top = data[candidate + 5u];
        if (data[candidate + 4u] == 0) {
            layers.count = data[candidate + 6u];
            layers.depth = data[candidate + 7u];
            for (var k = 0u; k < 4u; k++) {
                layers.under[k] = vec2<i32>(data[candidate + 8u + 2u * k], data[candidate + 9u + 2u * k]);
            }
        }
    }
    return layers;
}

// The block at `z` (`Layers::block_at`, the caves taken out as `CompiledMap::chunk` does): below the world
// stone, above it air, past the map air.
fn sight_block(column: vec4<i32>, layers: SightLayers, x: i32, y: i32, z: i32, caves: bool) -> i32 {
    let m = hu(H_MINERALOGY);
    let stone = data[m + 5u];
    let water = data[m + 6u];
    if (z < 0) {
        return stone;
    }
    let ground = column.x;
    if (ground < 0 || z >= 1024) {
        return 0;
    }
    var block = 0;
    if (z < ground - layers.depth) {
        block = stone;
    } else if (z < ground) {
        block = layers.top;
        for (var k = 0; k < layers.count; k++) {
            if (ground - z <= layers.under[k].y) {
                block = layers.under[k].x;
                break;
            }
        }
    } else if (z == ground) {
        block = layers.top;
    } else if (z < 128 || z < column.y) {
        return water;
    } else {
        return 0;
    }
    if (caves && block != 0 && block != water && cave_inside(x, y, z)) {
        return 0;
    }
    return block;
}

// 0 air, 1 water, 2 solid.
fn sight_kind(block: i32) -> i32 {
    if (block == 0) {
        return 0;
    }
    if (block == data[hu(H_MINERALOGY) + 6u]) {
        return 1;
    }
    return 2;
}

// Whether a cave box comes near a cell (its blocks or the ones just past its faces).
fn sight_cave_box(cx: i32, cy: i32, level: i32) -> bool {
    var slot = cave_slot(cx, cy, level);
    let mask = hu(H_CAVE_SLOTS) - 1u;
    for (var probe = 0u; probe <= mask; probe++) {
        let at = hu(H_CAVE_TABLE) + slot * 6u;
        if (data[at] == 0) {
            return false;
        }
        if (data[at + 1u] == cx && data[at + 2u] == cy && data[at + 3u] == level) {
            return true;
        }
        slot = (slot + 1u) & mask;
    }
    return false;
}

fn sight_near_caves(x0: i32, y0: i32, z0: i32, size: i32) -> bool {
    if (h(H_CAVE_SLOTS) == 0) {
        return false;
    }
    for (var level = div_floor(z0 - 1, 32); level <= div_floor(z0 + size, 32); level++) {
        for (var cx = div_floor(x0 - 1, 32); cx <= div_floor(x0 + size, 32); cx++) {
            for (var cy = div_floor(y0 - 1, 32); cy <= div_floor(y0 + size, 32); cy++) {
                if (sight_cave_box(cx, cy, level)) {
                    return true;
                }
            }
        }
    }
    return false;
}

// A face's colour: its block's top's looking up, its sides' every other way (`FarColours::colour`).
fn sight_colour(block: i32, up: bool, c: u32) -> u32 {
    if (block < 0 || block >= h(H_SIGHT_BLOCKS)) {
        return 511u;
    }
    var face = 3u;
    if (up) {
        face = 0u;
    }
    return u32(data[hu(H_SIGHT_COLOURS) + u32(block) * 6u + face + c]);
}

// A wide sum (two halves) with `value` added.
fn sight_wide(sums: ptr<function, array<u32, 17>>, at: u32, value: u32) {
    let lo = (*sums)[at] + value;
    if (lo < value) {
        (*sums)[at + 1u] = (*sums)[at + 1u] + 1u;
    }
    (*sums)[at] = lo;
}

// One column's blocks inside a cell, added to `sums`.
fn sight_column_sums(sums: ptr<function, array<u32, 17>>, x: i32, y: i32, cell: vec4<i32>, caves: bool, odd: bool) {
    let column = sight_column(x, y);
    if (column.x < 0 && !caves) {
        return;
    }
    let layers = sight_layers(column.z);
    // Beside, in `DIRECTIONS`' order after up and down: north, south, east, west.
    var beside: array<vec4<i32>, 4>;
    beside[0] = sight_column(x + 1, y);
    beside[1] = sight_column(x - 1, y);
    beside[2] = sight_column(x, y + 1);
    beside[3] = sight_column(x, y - 1);
    var beside_layers: array<SightLayers, 4>;
    for (var d = 0; d < 4; d++) {
        beside_layers[d] = sight_layers(beside[d].z);
    }
    var low = cell.z;
    var high = cell.z + cell.w - 1;
    if (!caves && !odd) {
        // Below its own ground and every neighbour's, solid all round: counted, not looked at.
        var floor = column.x;
        for (var d = 0; d < 4; d++) {
            floor = min(floor, beside[d].x);
        }
        let solid_to = min(high, floor - 1);
        if (solid_to >= low) {
            (*sums)[0] = (*sums)[0] + u32(solid_to - low + 1);
        }
        low = max(low, floor);
        // Above its own ground only water and air: the water counted.
        let water_top = min(max(128, column.y), 1024) - 1;
        let water_from = max(low, column.x + 1);
        let water_to = min(high, water_top);
        if (water_to >= water_from) {
            (*sums)[1] = (*sums)[1] + u32(water_to - water_from + 1);
        }
        high = min(high, column.x);
    }
    let offsets = array<vec2<i32>, 4>(vec2<i32>(1, 0), vec2<i32>(-1, 0), vec2<i32>(0, 1), vec2<i32>(0, -1));
    for (var z = low; z <= high; z++) {
        let block = sight_block(column, layers, x, y, z, caves);
        let kind = sight_kind(block);
        if (kind == 1) {
            (*sums)[1] = (*sums)[1] + 1u;
        }
        if (kind != 2) {
            continue;
        }
        (*sums)[0] = (*sums)[0] + 1u;
        for (var d = 0; d < 6; d++) {
            var next = 0;
            if (d == 0) {
                next = sight_block(column, layers, x, y, z + 1, caves);
            } else if (d == 1) {
                next = sight_block(column, layers, x, y, z - 1, caves);
            } else {
                let side = d - 2;
                next = sight_block(beside[side], beside_layers[side], x + offsets[side].x, y + offsets[side].y, z, caves);
            }
            if (sight_kind(next) == 2) {
                continue;
            }
            let r = sight_colour(block, d == 0, 0u);
            let g = sight_colour(block, d == 0, 1u);
            let b = sight_colour(block, d == 0, 2u);
            let light = (r * 2126u + g * 7152u + b * 722u + 5000u) / 10000u;
            let height = u32(max(z, 0));
            (*sums)[2u + u32(d)] = (*sums)[2u + u32(d)] + 1u;
            (*sums)[8] = (*sums)[8] + r;
            (*sums)[9] = (*sums)[9] + g;
            (*sums)[10] = (*sums)[10] + b;
            (*sums)[11] = (*sums)[11] + light;
            sight_wide(sums, 12u, light * light);
            (*sums)[14] = (*sums)[14] + height;
            sight_wide(sums, 15u, height * height);
        }
    }
}

// Each thread's sums, added together at the end.
var<workgroup> sight_part: array<array<u32, 17>, 64>;

// One cell to a group of 64 threads (so a big cell's work is shared, never one long thread holding the card),
// each taking every 64th of its columns.
@compute @workgroup_size(64)
fn sight_cells_pass(@builtin(workgroup_id) group: vec3<u32>, @builtin(local_invocation_index) t: u32, @builtin(num_workgroups) groups: vec3<u32>) {
    let i = group.x + group.y * groups.x;
    let count = arrayLength(&sight_cells);
    var sums: array<u32, 17>;
    if (i < count) {
        let cell = sight_cells[i];
        let size = cell.w;
        let caves = sight_near_caves(cell.x, cell.y, cell.z, size);
        let odd = data[hu(H_MINERALOGY) + 8u] != 0;
        for (var k = i32(t); k < size * size; k += 64) {
            sight_column_sums(&sums, cell.x + k % size, cell.y + k / size, cell, caves, odd);
        }
    }
    sight_part[t] = sums;
    workgroupBarrier();
    for (var stride = 32u; stride > 0u; stride = stride / 2u) {
        if (t < stride) {
            var mine = sight_part[t];
            let other = sight_part[t + stride];
            for (var k = 0u; k < 17u; k++) {
                if (k == 12u || k == 15u) {
                    sight_wide(&mine, k, other[k]);
                    mine[k + 1u] = mine[k + 1u] + other[k + 1u];
                    k = k + 1u;
                } else {
                    mine[k] = mine[k] + other[k];
                }
            }
            sight_part[t] = mine;
        }
        workgroupBarrier();
    }
    if (t == 0u && i < count) {
        for (var k = 0u; k < SIGHT_SUMS; k++) {
            sight_sums[i * SIGHT_SUMS + k] = sight_part[0][k];
        }
    }
}

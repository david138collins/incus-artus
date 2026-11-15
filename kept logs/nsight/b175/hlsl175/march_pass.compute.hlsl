struct NagaConstants {
    int first_vertex;
    int first_instance;
    uint other;
};
ConstantBuffer<NagaConstants> _NagaConstants: register(b1);

struct Camera {
    float4 position;
    float4 forward;
    float4 right;
    float4 up;
    float4 view;
    float4 sun;
    float4 reach;
    float4 tiles;
    float4 layer;
    float4 follow;
    float4 pyramid;
    float4 fade;
    float4 sky;
    float4 fog;
    float4 daylight;
    float4 water;
    float4 atlas;
    float4 window;
    float4 ray_start;
    float4 rings[4];
};

struct Seen {
    float3 colour;
    float distance_;
    float steps;
    int level;
    int _pad4_0;
    int _pad4_1;
    float3 sunny;
    int _pad5_0;
    float3 at;
    float wide;
    float3 skyish;
    float touched;
    float sky_share;
    int _end_pad_0;
    int _end_pad_1;
    int _end_pad_2;
};

struct Look {
    bool known;
    float solid;
    float water;
    int _pad3_0;
    float3 colour;
    int _pad4_0;
    float3 plus;
    int _pad5_0;
    float3 minus;
    float surface;
};

struct Place {
    int2 region;
    bool known;
    bool summed;
    uint usable;
};

struct Row {
    int level;
    int _pad1_0;
    int2 least;
    int across;
    int entries_at;
};

struct Kept {
    Row row;
    int index_at;
    uint wide;
    uint tall;
    int bricks;
    int bricks_up;
    int up;
    int3 brick;
    uint slot;
};

struct Spot {
    int kind;
    int level;
    int _pad2_0;
    int _pad2_1;
    float3 at;
    int _end_pad_0;
};

struct Fetched {
    float2 sw;
    int _pad1_0;
    int _pad1_1;
    float4 c;
    float3 plus;
    int _pad3_0;
    float3 minus;
    int _end_pad_0;
};

static const int REGION = int(10);
static const int FIRST = int(5);
static const int TOP = int(16);
static const int TABLE = int(12);
static const float WORLD_HEIGHT = 1024.0;
static const uint NO_SLOT = 4294967295u;
static const uint PAST_MAP = 4294836224u;
static const uint NOT_SUMMED = 4294901760u;
static const float RING_FADE = 8.0;
static const float3 ROCK = float3(0.25, 0.24, 0.22);
static const uint NO_BRICK = 4294967295u;
static const float3 NIGHT_SKY = float3(0.35, 0.45, 0.8);
static const float3 DUSK = float3(1.0, 0.45, 0.18);
static const float3 HORIZON = float3(0.62, 0.72, 0.86);
static const float3 ZENITH = float3(0.16, 0.32, 0.66);
static const float3 SKY_TINT = float3(0.84, 1.01, 1.4);
static const float3 FOG_WARM = float3(1.0, 0.82, 0.62);

cbuffer camera : register(b0) { Camera camera; }
ByteAddressBuffer levels : register(t0);
ByteAddressBuffer entries : register(t1);
ByteAddressBuffer index : register(t2);
ByteAddressBuffer pool : register(t3);
RWByteAddressBuffer starts : register(u2);
RWTexture2D<unorm float4> picture : register(u0);
RWTexture2D<float4> how_far : register(u1);
ByteAddressBuffer worth_in : register(t4);
RWByteAddressBuffer worth_out : register(u3);
Texture2D<float4> far_above : register(t5);
ByteAddressBuffer slots : register(t6);
Texture3D<float4> t_solid : register(t7);
SamplerState nagaSamplerHeap[2048]: register(s0, space0);
SamplerComparisonState nagaComparisonSamplerHeap[2048]: register(s2048, space0);
StructuredBuffer<uint> nagaGroup0SamplerIndexArray : register(t14, space0);
static const SamplerState soft = nagaSamplerHeap[nagaGroup0SamplerIndexArray[0]];
Texture3D<float4> t_colour : register(t8);
Texture3D<float4> t_plus : register(t9);
Texture3D<float4> t_minus : register(t10);
Texture2D<float4> t_blue : register(t13);
Texture2D<float4> t_shadow : register(t11);
Texture2D<float4> t_sky : register(t12);

void put_picture(int2 at, float4 v)
{
    picture[at] = v;
    return;
}

void put_far(int2 at_1, float4 v_1)
{
    how_far[at_1] = v_1;
    return;
}

void put_worth(uint i, uint v_2)
{
    worth_out.Store(i*4, asuint(v_2));
    return;
}

uint get_worth(uint i_1)
{
    uint _e3 = asuint(worth_in.Load(i_1*4));
    return _e3;
}

uint2 NagaDimensions2D(Texture2D<float4> tex)
{
    uint4 ret;
    tex.GetDimensions(0, ret.x, ret.y, ret.z);
    return ret.xy;
}

int2 far_above_edge()
{
    return asint(asuint(int2(NagaDimensions2D(far_above))) - asuint((int(1)).xx));
}

float get_far_above(int2 at_2)
{
    float4 _e3 = far_above.Load(int3(at_2, int(0)));
    return _e3.x;
}

int table(int level, int k)
{
    int row_4 = ((level < FIRST) ? asint(asuint(int(12)) + asuint(asint(asuint(int(4)) - asuint(level)))) : asint(asuint(level) - asuint(FIRST)));
    int _e16 = asint(levels.Load(asint(asuint(asint(asuint(row_4) * asuint(TABLE))) + asuint(k))*4));
    return _e16;
}

int naga_f2i32(float value) {
    return int(clamp(value, -2147483600.0, 2147483500.0));
}

int level_at(float t)
{
    float _e4 = camera.view.z;
    float footprint_1 = max((t * _e4), 1.0);
    float _e14 = camera.reach.y;
    int level_15 = asint(asuint(naga_f2i32(floor(log2(footprint_1)))) + asuint(naga_f2i32(_e14)));
    float _e20 = camera.follow.z;
    return clamp(level_15, naga_f2i32(_e20), TOP);
}

float leaves(float2 o, float2 d, float2 lo, float2 hi)
{
    float2 to = ((d > (0.0).xx) ? hi : lo);
    float2 t_3 = ((to - o) / ((abs(d) < (1e-9).xx) ? (1e-9).xx : d));
    return min(((abs(d.x) < 1e-9) ? 1e30 : t_3.x), ((abs(d.y) < 1e-9) ? 1e30 : t_3.y));
}

float map_end(float3 o_1, float3 d_1)
{
    bool local = (bool)0;

    const int _e5 = table(REGION, int(1));
    const int _e9 = table(REGION, int(2));
    float2 lo_1 = (float2(float(_e5), float(_e9)) * 1024.0);
    const int _e15 = table(REGION, int(3));
    float2 hi_1 = (lo_1 + ((float(_e15) * 1024.0)).xx);
    if (!(any((o_1.xy < lo_1)))) {
        local = any((o_1.xy > hi_1));
    } else {
        local = true;
    }
    bool _e30 = local;
    if (_e30) {
        float _e34 = camera.reach.x;
        return _e34;
    }
    const float _e37 = leaves(o_1.xy, d_1.xy, lo_1, hi_1);
    const int _e38 = level_at(_e37);
    float s = (2.0 * float((int(1) << uint(min(asint(asuint(_e38) + asuint(int(1))), TOP)))));
    const float _e55 = leaves(o_1.xy, d_1.xy, (lo_1 - (s).xx), (hi_1 + (s).xx));
    return _e55;
}

float world_top()
{
    float _e3 = camera.reach.w;
    return min(max(_e3, 0.0), WORLD_HEIGHT);
}

Row ConstructRow(int arg0, int2 arg1, int arg2, int arg3) {
    Row ret = (Row)0;
    ret.level = arg0;
    ret.least = arg1;
    ret.across = arg2;
    ret.entries_at = arg3;
    return ret;
}

Row row_none()
{
    const Row row_5 = ConstructRow(int(-1), (int(0)).xx, int(0), int(0));
    return row_5;
}

Kept ConstructKept(Row arg0, int arg1, uint arg2, uint arg3, int arg4, int arg5, int arg6, int3 arg7, uint arg8) {
    Kept ret = (Kept)0;
    ret.row = arg0;
    ret.index_at = arg1;
    ret.wide = arg2;
    ret.tall = arg3;
    ret.bricks = arg4;
    ret.bricks_up = arg5;
    ret.up = arg6;
    ret.brick = arg7;
    ret.slot = arg8;
    return ret;
}

Kept kept_none()
{
    const Row _e0 = row_none();
    const Kept kept_1 = ConstructKept(_e0, int(0), 0u, 0u, int(0), int(0), int(0), (int(-1)).xxx, NO_SLOT);
    return kept_1;
}

float in_ring(int level_1, float3 p)
{
    float4 box = camera.rings[asint(asuint(int(4)) - asuint(level_1))];
    return min(min((p.x - box.x), (box.z - p.x)), min((p.y - box.y), (box.w - p.y)));
}

int ring_level(int level_2, float3 p_1)
{
    int l = (int)0;
    bool local_1 = (bool)0;

    l = level_2;
    uint2 loop_bound = uint2(4294967295u, 4294967295u);
    while(true) {
        if (all(loop_bound == uint2(0u, 0u))) { break; }
        loop_bound -= uint2(loop_bound.y == 0u, 1u);
        int _e3 = l;
        if ((_e3 < FIRST)) {
            int _e8 = l;
            const float _e9 = in_ring(_e8, p_1);
            local_1 = (_e9 < 0.0);
        } else {
            local_1 = false;
        }
        bool _e13 = local_1;
        if (_e13) {
        } else {
            break;
        }
        {
            int _e14 = l;
            l = asint(asuint(_e14) + asuint(int(1)));
        }
    }
    int _e17 = l;
    return _e17;
}

uint entry_at(int level_3, int cx, int cy)
{
    bool local_2 = (bool)0;
    bool local_3 = (bool)0;
    bool local_4 = (bool)0;

    const int _e4 = table(level_3, int(1));
    int x = asint(asuint(cx) - asuint(_e4));
    const int _e7 = table(level_3, int(2));
    int y = asint(asuint(cy) - asuint(_e7));
    const int _e10 = table(level_3, int(3));
    if (!((x < int(0)))) {
        local_2 = (y < int(0));
    } else {
        local_2 = true;
    }
    bool _e19 = local_2;
    if (!(_e19)) {
        local_3 = (x >= _e10);
    } else {
        local_3 = true;
    }
    bool _e25 = local_3;
    if (!(_e25)) {
        local_4 = (y >= _e10);
    } else {
        local_4 = true;
    }
    bool _e31 = local_4;
    if (_e31) {
        return PAST_MAP;
    }
    const int _e35 = table(level_3, int(4));
    uint _e41 = asuint(entries.Load(uint(asint(asuint(asint(asuint(_e35) + asuint(asint(asuint(y) * asuint(_e10))))) + asuint(x)))*4));
    return _e41;
}

Place ConstructPlace(int2 arg0, bool arg1, bool arg2, uint arg3) {
    Place ret = (Place)0;
    ret.region = arg0;
    ret.known = arg1;
    ret.summed = arg2;
    ret.usable = arg3;
    return ret;
}

int2 naga_f2i32(float2 value) {
    return int2(clamp(value, -2147483600.0, 2147483500.0));
}

void keep_place(inout Place place, float3 p_2)
{
    bool local_5 = (bool)0;
    uint usable = 0u;
    bool local_6 = (bool)0;
    bool local_7 = (bool)0;
    bool local_8 = (bool)0;
    int k_1 = int(0);
    bool local_9 = (bool)0;

    int2 r = naga_f2i32(floor((p_2.xy / (1024.0).xx)));
    bool _e9 = place.known;
    if (_e9) {
        int2 _e13 = place.region;
        local_5 = all((r == _e13));
    } else {
        local_5 = false;
    }
    bool _e17 = local_5;
    if (_e17) {
        return;
    }
    float _e23 = camera.window.y;
    int n_2 = naga_f2i32(_e23);
    float4 _e27 = camera.window;
    int2 rr = asint(asuint(r) - asuint(naga_f2i32(_e27.zw)));
    if ((rr.x >= int(0))) {
        local_6 = (rr.y >= int(0));
    } else {
        local_6 = false;
    }
    bool _e40 = local_6;
    if (_e40) {
        local_7 = (rr.x < n_2);
    } else {
        local_7 = false;
    }
    bool _e46 = local_7;
    if (_e46) {
        local_8 = (rr.y < n_2);
    } else {
        local_8 = false;
    }
    bool on_map = local_8;
    const int _e55 = table(TOP, int(4));
    const int _e58 = table(TOP, int(3));
    const int _e61 = table(TOP, int(3));
    int flags_at = asint(asuint(_e55) + asuint(asint(asuint(_e58) * asuint(_e61))));
    uint2 loop_bound_1 = uint2(4294967295u, 4294967295u);
    bool loop_init = true;
    while(true) {
        if (all(loop_bound_1 == uint2(0u, 0u))) { break; }
        loop_bound_1 -= uint2(loop_bound_1.y == 0u, 1u);
        if (!loop_init) {
            int _e100 = k_1;
            k_1 = asint(asuint(_e100) + asuint(int(1)));
        }
        loop_init = false;
        int _e66 = k_1;
        float _e70 = camera.window.x;
        if ((_e66 < naga_f2i32(_e70))) {
        } else {
            break;
        }
        {
            if (!(!(on_map))) {
                int _e78 = k_1;
                uint _e88 = asuint(entries.Load(uint(asint(asuint(asint(asuint(flags_at) + asuint(asint(asuint(asint(asuint(asint(asuint(_e78) * asuint(n_2))) + asuint(rr.y))) * asuint(n_2))))) + asuint(rr.x)))*4));
                local_9 = (_e88 != 0u);
            } else {
                local_9 = true;
            }
            bool _e92 = local_9;
            if (_e92) {
                uint _e93 = usable;
                int _e95 = k_1;
                usable = (_e93 | (1u << uint(_e95)));
            }
        }
    }
    const uint _e106 = entry_at(REGION, r.x, r.y);
    uint _e109 = usable;
    place = ConstructPlace(r, true, (_e106 != NOT_SUMMED), _e109);
    return;
}

float leave2_(float2 o_2, float2 d_2, float2 c, float size)
{
    float2 next_1 = ((c + ((d_2 > (0.0).xx) ? (1.0).xx : (0.0).xx)) * size);
    float2 t_4 = ((next_1 - o_2) / d_2);
    float tx = ((abs(d_2.x) > 1e-6) ? t_4.x : 1e30);
    float ty = ((abs(d_2.y) > 1e-6) ? t_4.y : 1e30);
    return min(tx, ty);
}

void keep_row(inout Row row, int level_4)
{
    int _e3 = row.level;
    if ((_e3 != level_4)) {
        const int _e6 = table(level_4, int(1));
        const int _e8 = table(level_4, int(2));
        const int _e11 = table(level_4, int(3));
        const int _e13 = table(level_4, int(4));
        row = ConstructRow(level_4, int2(_e6, _e8), _e11, _e13);
        return;
    } else {
        return;
    }
}

uint naga_f2u32(float value) {
    return uint(clamp(value, 0.0, 4294967000.0));
}

uint top_in(Row row_1, int cx_1, int cy_1)
{
    bool local_10 = (bool)0;
    bool local_11 = (bool)0;
    bool local_12 = (bool)0;

    int x_1 = asint(asuint(cx_1) - asuint(row_1.least.x));
    int y_1 = asint(asuint(cy_1) - asuint(row_1.least.y));
    if (!((x_1 < int(0)))) {
        local_10 = (y_1 < int(0));
    } else {
        local_10 = true;
    }
    bool _e17 = local_10;
    if (!(_e17)) {
        local_11 = (x_1 >= row_1.across);
    } else {
        local_11 = true;
    }
    bool _e24 = local_11;
    if (!(_e24)) {
        local_12 = (y_1 >= row_1.across);
    } else {
        local_12 = true;
    }
    bool _e31 = local_12;
    if (_e31) {
        return 4294967295u;
    }
    float _e43 = camera.pyramid.w;
    uint _e47 = asuint(entries.Load((uint(asint(asuint(asint(asuint(row_1.entries_at) + asuint(asint(asuint(y_1) * asuint(row_1.across))))) + asuint(x_1))) + naga_f2u32(_e43))*4));
    return _e47;
}

void keep_level(inout Kept k_2, int level_5)
{
    int _e4 = k_2.row.level;
    if ((_e4 != level_5)) {
        const int _e7 = table(level_5, int(1));
        const int _e9 = table(level_5, int(2));
        const int _e12 = table(level_5, int(3));
        const int _e14 = table(level_5, int(4));
        const int _e17 = table(level_5, int(5));
        const int _e19 = table(level_5, int(6));
        const int _e22 = table(level_5, int(7));
        const int _e25 = table(level_5, int(8));
        const int _e27 = table(level_5, int(9));
        const int _e29 = table(level_5, int(10));
        k_2 = ConstructKept(ConstructRow(level_5, int2(_e7, _e9), _e12, _e14), _e17, uint(_e19), uint(_e22), _e25, _e27, _e29, (int(-1)).xxx, NO_SLOT);
        return;
    } else {
        return;
    }
}

uint entry_in(Row row_2, int cx_2, int cy_2)
{
    bool local_13 = (bool)0;
    bool local_14 = (bool)0;
    bool local_15 = (bool)0;

    int x_2 = asint(asuint(cx_2) - asuint(row_2.least.x));
    int y_2 = asint(asuint(cy_2) - asuint(row_2.least.y));
    if (!((x_2 < int(0)))) {
        local_13 = (y_2 < int(0));
    } else {
        local_13 = true;
    }
    bool _e17 = local_13;
    if (!(_e17)) {
        local_14 = (x_2 >= row_2.across);
    } else {
        local_14 = true;
    }
    bool _e24 = local_14;
    if (!(_e24)) {
        local_15 = (y_2 >= row_2.across);
    } else {
        local_15 = true;
    }
    bool _e31 = local_15;
    if (_e31) {
        return PAST_MAP;
    }
    uint _e41 = asuint(entries.Load(uint(asint(asuint(asint(asuint(row_2.entries_at) + asuint(asint(asuint(y_2) * asuint(row_2.across))))) + asuint(x_2)))*4));
    return _e41;
}

Spot ConstructSpot(int arg0, int arg1, float3 arg2) {
    Spot ret = (Spot)0;
    ret.kind = arg0;
    ret.level = arg1;
    ret.at = arg2;
    return ret;
}

int3 naga_f2i32(float3 value) {
    return int3(clamp(value, -2147483600.0, 2147483500.0));
}

Spot spot_soft(inout Kept k_3, Place place_1, int level_6, float3 p_3)
{
    bool inside = false;
    bool local_16 = (bool)0;
    bool local_17 = (bool)0;
    int n = int(0);
    bool local_18 = (bool)0;
    bool local_19 = (bool)0;
    bool local_20 = (bool)0;
    bool local_21 = (bool)0;
    bool local_22 = (bool)0;

    float size_1 = float((int(1) << uint(level_6)));
    float3 u = ((p_3 / (size_1).xxx) - (0.5).xxx);
    float3 b_2 = floor(u);
    float3 f = (u - b_2);
    int3 base = naga_f2i32(b_2);
    Row row_6 = k_3.row;
    if ((level_6 <= REGION)) {
        local_16 = place_1.summed;
    } else {
        local_16 = false;
    }
    bool _e26 = local_16;
    if (_e26) {
        uint2 shift = (uint(asint(asuint(REGION) - asuint(level_6)))).xx;
        if (all(((base.xy >> shift) == place_1.region))) {
            local_17 = all(((asint(asuint(base.xy) + asuint((int(1)).xx)) >> shift) == place_1.region));
        } else {
            local_17 = false;
        }
        bool _e47 = local_17;
        inside = _e47;
    }
    bool _e48 = inside;
    if (!(_e48)) {
        uint2 loop_bound_2 = uint2(4294967295u, 4294967295u);
        bool loop_init_1 = true;
        while(true) {
            if (all(loop_bound_2 == uint2(0u, 0u))) { break; }
            loop_bound_2 -= uint2(loop_bound_2.y == 0u, 1u);
            if (!loop_init_1) {
                int _e73 = n;
                n = asint(asuint(_e73) + asuint(int(1)));
            }
            loop_init_1 = false;
            int _e52 = n;
            if ((_e52 < int(4))) {
            } else {
                break;
            }
            {
                int _e56 = n;
                int _e61 = n;
                const uint _e65 = entry_in(row_6, asint(asuint(base.x) + asuint((_e56 & int(1)))), asint(asuint(base.y) + asuint((_e61 >> 1u))));
                if ((_e65 == NOT_SUMMED)) {
                    const Spot spot_1 = ConstructSpot(int(0), level_6, (0.0).xxx);
                    return spot_1;
                }
            }
        }
    }
    int x_3 = asint(asuint(base.x) - asuint(row_6.least.x));
    int y_3 = asint(asuint(base.y) - asuint(row_6.least.y));
    if (!((x_3 < int(0)))) {
        local_18 = (y_3 < int(0));
    } else {
        local_18 = true;
    }
    bool _e91 = local_18;
    if (!(_e91)) {
        local_19 = (base.z < int(0));
    } else {
        local_19 = true;
    }
    bool _e99 = local_19;
    if (!(_e99)) {
        local_20 = (x_3 >= row_6.across);
    } else {
        local_20 = true;
    }
    bool _e106 = local_20;
    if (!(_e106)) {
        local_21 = (y_3 >= row_6.across);
    } else {
        local_21 = true;
    }
    bool _e113 = local_21;
    if (!(_e113)) {
        int _e119 = k_3.up;
        local_22 = (base.z >= _e119);
    } else {
        local_22 = true;
    }
    bool _e122 = local_22;
    if (_e122) {
        const Spot spot_2 = ConstructSpot(int(1), level_6, (0.0).xxx);
        return spot_2;
    }
    uint wide_1 = k_3.wide;
    uint tall = k_3.tall;
    int3 brick = int3(int((uint(x_3) >> wide_1)), int((uint(y_3) >> wide_1)), int((uint(base.z) >> tall)));
    int3 _e143 = k_3.brick;
    if (any((brick != _e143))) {
        k_3.brick = brick;
        int _e150 = k_3.index_at;
        int _e153 = k_3.bricks;
        int _e158 = k_3.bricks_up;
        uint _e165 = asuint(slots.Load(uint(asint(asuint(asint(asuint(_e150) + asuint(asint(asuint(asint(asuint(asint(asuint(brick.y) * asuint(_e153))) + asuint(brick.x))) * asuint(_e158))))) + asuint(brick.z)))*4));
        k_3.slot = _e165;
    }
    uint slot = k_3.slot;
    if ((slot == NO_SLOT)) {
        const Spot spot_3 = ConstructSpot(int(1), level_6, (0.0).xxx);
        return spot_3;
    }
    float _e189 = camera.atlas.w;
    float3 origin = (float3(float((slot & 255u)), float(((slot >> 8u) & 255u)), float((slot >> 16u))) * _e189);
    float3 local_50 = float3(float((uint(x_3) & ((1u << wide_1) - 1u))), float((uint(y_3) & ((1u << wide_1) - 1u))), float((uint(base.z) & ((1u << tall) - 1u))));
    float4 _e221 = camera.atlas;
    const Spot spot_4 = ConstructSpot(int(2), level_6, ((((origin + local_50) + f) + (0.5).xxx) / _e221.xyz));
    return spot_4;
}

Spot spot_kept(inout Kept k_4, Place place_2, int level_7, float3 p_4)
{
    int l_1 = (int)0;
    bool local_23 = (bool)0;
    bool local_24 = (bool)0;

    l_1 = level_7;
    int _e5 = l_1;
    const int _e6 = ring_level(_e5, p_4);
    l_1 = _e6;
    uint2 loop_bound_3 = uint2(4294967295u, 4294967295u);
    while(true) {
        if (all(loop_bound_3 == uint2(0u, 0u))) { break; }
        loop_bound_3 -= uint2(loop_bound_3.y == 0u, 1u);
        int _e7 = l_1;
        if ((_e7 >= FIRST)) {
            int _e12 = l_1;
            float _e18 = camera.window.x;
            local_23 = (asint(asuint(_e12) - asuint(FIRST)) < naga_f2i32(_e18));
        } else {
            local_23 = false;
        }
        bool _e22 = local_23;
        if (_e22) {
            int _e27 = l_1;
            local_24 = ((place_2.usable & (1u << uint(asint(asuint(_e27) - asuint(FIRST))))) == 0u);
        } else {
            local_24 = false;
        }
        bool _e36 = local_24;
        if (_e36) {
        } else {
            break;
        }
        {
            int _e37 = l_1;
            l_1 = asint(asuint(_e37) + asuint(int(1)));
        }
    }
    float _e43 = camera.layer.w;
    if ((_e43 < 0.5)) {
        int _e46 = l_1;
        const Spot spot_5 = ConstructSpot(int(1), _e46, (0.0).xxx);
        return spot_5;
    }
    int _e51 = l_1;
    keep_level(k_4, _e51);
    int _e52 = l_1;
    const Spot _e53 = spot_soft(k_4, place_2, _e52, p_4);
    const Spot spot_6 = _e53;
    return spot_6;
}

Fetched ConstructFetched(float2 arg0, float4 arg1, float3 arg2, float3 arg3) {
    Fetched ret = (Fetched)0;
    ret.sw = arg0;
    ret.c = arg1;
    ret.plus = arg2;
    ret.minus = arg3;
    return ret;
}

Fetched fetch_together(float3 at_3)
{
    float4 _e4 = t_solid.SampleLevel(soft, at_3, 0.0);
    float4 _e9 = t_colour.SampleLevel(soft, at_3, 0.0);
    float4 _e13 = t_plus.SampleLevel(soft, at_3, 0.0);
    float4 _e18 = t_minus.SampleLevel(soft, at_3, 0.0);
    const Fetched fetched = ConstructFetched(_e4.xy, _e9, _e13.xyz, _e18.xyz);
    return fetched;
}

Look empty_look(bool known)
{
    Look look = (Look)0;

    look.known = known;
    Look _e3 = look;
    const Look look_6 = _e3;
    return look_6;
}

Look look_of(Fetched got)
{
    Look look_1 = (Look)0;

    const Look _e2 = empty_look(true);
    look_1 = _e2;
    look_1.solid = got.sw.x;
    look_1.water = got.sw.y;
    look_1.colour = ROCK;
    if (((got.sw.x + got.sw.y) > 0.0)) {
        look_1.colour = got.c.xyz;
        look_1.plus = got.plus;
        look_1.minus = got.minus;
        look_1.surface = (got.c.w * 4.0);
    }
    Look _e31 = look_1;
    const Look look_7 = _e31;
    return look_7;
}

Look mix_looks(Look a, Look b, float w)
{
    Look out_ = (Look)0;

    float k_9 = clamp(w, 0.0, 1.0);
    out_ = a;
    out_.solid = lerp(a.solid, b.solid, k_9);
    out_.water = lerp(a.water, b.water, k_9);
    out_.colour = lerp(a.colour, b.colour, k_9);
    out_.plus = lerp(a.plus, b.plus, k_9);
    out_.minus = lerp(a.minus, b.minus, k_9);
    out_.surface = lerp(a.surface, b.surface, k_9);
    Look _e31 = out_;
    const Look look_8 = _e31;
    return look_8;
}

Look solid_look()
{
    Look look_2 = (Look)0;

    look_2.known = true;
    look_2.solid = 1.0;
    Look _e5 = look_2;
    const Look look_9 = _e5;
    return look_9;
}

Look cell_of(uint4 w_1)
{
    Look look_3 = (Look)0;

    look_3.known = true;
    look_3.solid = (float((w_1.x & 255u)) / 255.0);
    look_3.water = (float(((w_1.x >> 8u) & 255u)) / 255.0);
    float up = float(((w_1.x >> 16u) & 255u));
    float down = float((w_1.x >> 24u));
    float north = float((w_1.y & 255u));
    float south = float(((w_1.y >> 8u) & 255u));
    float east = float(((w_1.y >> 16u) & 255u));
    float west = max((((((255.0 - up) - down) - north) - south) - east), 0.0);
    look_3.plus = (float3(north, east, up) / (255.0).xxx);
    look_3.minus = (float3(south, west, down) / (255.0).xxx);
    look_3.colour = (float3(float(((w_1.z >> 20u) & 1023u)), float(((w_1.z >> 10u) & 1023u)), float((w_1.z & 1023u))) / (1023.0).xxx);
    look_3.surface = min((float((w_1.w >> 16u)) / 256.0), 4.0);
    Look _e94 = look_3;
    const Look look_10 = _e94;
    return look_10;
}

uint4 cell_word(int level_8, int cx_3, int cy_3, int cz)
{
    const int _e5 = table(level_8, int(6));
    uint wide_2 = uint(_e5);
    const int _e8 = table(level_8, int(7));
    uint tall_1 = uint(_e8);
    const int _e11 = table(level_8, int(1));
    uint x_4 = uint(asint(asuint(cx_3) - asuint(_e11)));
    const int _e15 = table(level_8, int(2));
    uint y_4 = uint(asint(asuint(cy_3) - asuint(_e15)));
    uint z = uint(cz);
    const int _e22 = table(level_8, int(8));
    const int _e28 = table(level_8, int(9));
    int brick_1 = asint(asuint(asint(asuint(asint(asuint(asint(asuint(int((y_4 >> wide_2))) * asuint(_e22))) + asuint(int((x_4 >> wide_2))))) * asuint(_e28))) + asuint(int((z >> tall_1))));
    const int _e35 = table(level_8, int(5));
    uint start = asuint(index.Load(uint(asint(asuint(_e35) + asuint(brick_1)))*4));
    if ((start == NO_BRICK)) {
        return (0u).xxxx;
    }
    uint mask = ((1u << wide_2) - 1u);
    uint4 _e64 = asuint(pool.Load4((((start + (x_4 & mask)) + ((y_4 & mask) << wide_2)) + ((z & ((1u << tall_1) - 1u)) << (2u * wide_2)))*16));
    return _e64;
}

Look from_band(int level_9, int cx_4, int cy_4, uint e, int cz_1)
{
    bool local_25 = (bool)0;
    bool local_26 = (bool)0;
    bool local_27 = (bool)0;

    if ((e == NOT_SUMMED)) {
        const Look _e8 = empty_look(false);
        const Look look_11 = _e8;
        return look_11;
    }
    if ((e == PAST_MAP)) {
        const Look _e12 = empty_look(true);
        const Look look_12 = _e12;
        return look_12;
    }
    int low = (asint((e << 16u)) >> 16u);
    int count = int((e >> 16u));
    if (!((cz_1 < int(0)))) {
        if ((cz_1 < low)) {
            local_26 = (low > int(0));
        } else {
            local_26 = false;
        }
        bool _e32 = local_26;
        local_25 = _e32;
    } else {
        local_25 = true;
    }
    bool _e34 = local_25;
    if (_e34) {
        const Look _e35 = solid_look();
        const Look look_13 = _e35;
        return look_13;
    }
    if (!((cz_1 < low))) {
        local_27 = (cz_1 >= asint(asuint(low) + asuint(count)));
    } else {
        local_27 = true;
    }
    bool _e43 = local_27;
    if (_e43) {
        const Look _e45 = empty_look(true);
        const Look look_14 = _e45;
        return look_14;
    }
    const uint4 _e46 = cell_word(level_9, cx_4, cy_4, cz_1);
    const Look _e47 = cell_of(_e46);
    const Look look_15 = _e47;
    return look_15;
}

Look blend(int level_10, float3 p_5)
{
    Look out_1 = (Look)0;
    float weight = 0.0;
    int k_5 = int(0);
    int oz = (int)0;

    float size_2 = float((int(1) << uint(level_10)));
    float3 u_1 = ((p_5 / (size_2).xxx) - (0.5).xxx);
    float3 b_3 = floor(u_1);
    float3 f_1 = (u_1 - b_3);
    int3 base_1 = naga_f2i32(b_3);
    const Look _e15 = empty_look(true);
    out_1 = _e15;
    uint2 loop_bound_4 = uint2(4294967295u, 4294967295u);
    bool loop_init_2 = true;
    while(true) {
        if (all(loop_bound_4 == uint2(0u, 0u))) { break; }
        loop_bound_4 -= uint2(loop_bound_4.y == 0u, 1u);
        if (!loop_init_2) {
            int _e105 = k_5;
            k_5 = asint(asuint(_e105) + asuint(int(1)));
        }
        loop_init_2 = false;
        int _e21 = k_5;
        if ((_e21 < int(4))) {
        } else {
            break;
        }
        {
            int _e24 = k_5;
            int ox = (_e24 & int(1));
            int _e27 = k_5;
            int oy = (_e27 >> 1u);
            int cx_5 = asint(asuint(base_1.x) + asuint(ox));
            int cy_5 = asint(asuint(base_1.y) + asuint(oy));
            const uint _e34 = entry_at(level_10, cx_5, cy_5);
            if ((_e34 == NOT_SUMMED)) {
                const Look _e38 = empty_look(false);
                const Look look_16 = _e38;
                return look_16;
            }
            float wxy = (((ox == int(1)) ? f_1.x : (1.0 - f_1.x)) * ((oy == int(1)) ? f_1.y : (1.0 - f_1.y)));
            oz = int(0);
            uint2 loop_bound_5 = uint2(4294967295u, 4294967295u);
            bool loop_init_3 = true;
            while(true) {
                if (all(loop_bound_5 == uint2(0u, 0u))) { break; }
                loop_bound_5 -= uint2(loop_bound_5.y == 0u, 1u);
                if (!loop_init_3) {
                    int _e102 = oz;
                    oz = asint(asuint(_e102) + asuint(int(1)));
                }
                loop_init_3 = false;
                int _e56 = oz;
                if ((_e56 < int(2))) {
                } else {
                    break;
                }
                {
                    int _e60 = oz;
                    const Look _e62 = from_band(level_10, cx_5, cy_5, _e34, asint(asuint(base_1.z) + asuint(_e60)));
                    int _e67 = oz;
                    float w_2 = (wxy * ((_e67 == int(1)) ? f_1.z : (1.0 - f_1.z)));
                    float _e73 = out_1.solid;
                    out_1.solid = (_e73 + (w_2 * _e62.solid));
                    float _e78 = out_1.water;
                    out_1.water = (_e78 + (w_2 * _e62.water));
                    float shown_1 = (w_2 * _e62.surface);
                    float3 _e85 = out_1.colour;
                    out_1.colour = (_e85 + (shown_1 * _e62.colour));
                    float3 _e90 = out_1.plus;
                    out_1.plus = (_e90 + (shown_1 * _e62.plus));
                    float3 _e95 = out_1.minus;
                    out_1.minus = (_e95 + (shown_1 * _e62.minus));
                    float _e99 = weight;
                    weight = (_e99 + shown_1);
                }
            }
        }
    }
    float _e107 = weight;
    if ((_e107 > 0.0)) {
        float3 _e111 = out_1.colour;
        float _e112 = weight;
        out_1.colour = (_e111 / (_e112).xxx);
        float3 _e116 = out_1.plus;
        float _e117 = weight;
        out_1.plus = (_e116 / (_e117).xxx);
        float3 _e121 = out_1.minus;
        float _e122 = weight;
        out_1.minus = (_e121 / (_e122).xxx);
        float _e126 = weight;
        out_1.surface = _e126;
    } else {
        out_1.colour = ROCK;
    }
    Look _e129 = out_1;
    const Look look_17 = _e129;
    return look_17;
}

Look look_at(Spot spot, float3 p_6)
{
    if ((spot.kind == int(0))) {
        const Look _e6 = empty_look(false);
        const Look look_18 = _e6;
        return look_18;
    }
    if ((spot.kind == int(1))) {
        const Look _e11 = blend(spot.level, p_6);
        const Look look_19 = _e11;
        return look_19;
    }
    const Fetched _e13 = fetch_together(spot.at);
    const Look _e14 = look_of(_e13);
    const Look look_20 = _e14;
    return look_20;
}

Look far_blend_kept(inout Kept k_6, Place place_3, int level_11, float3 p_7)
{
    const Spot _e4 = spot_kept(k_6, place_3, level_11, p_7);
    const Look _e5 = look_at(_e4, p_7);
    const Look look_21 = _e5;
    return look_21;
}

float faces_light(Look look_4, float3 d_3)
{
    float light = 0.0;

    float4 _e4 = camera.sun;
    float3 sun_1 = _e4.xyz;
    float3 seen_plus = (look_4.plus * max(-(d_3), (0.0).xxx));
    float3 seen_minus = (look_4.minus * max(d_3, (0.0).xxx));
    float seen_1 = dot((seen_plus + seen_minus), (1.0).xxx);
    float3 sunny_plus = max(sun_1, (0.0).xxx);
    float3 sunny_minus = max(-(sun_1), (0.0).xxx);
    if ((seen_1 > 0.0001)) {
        light = ((dot(seen_plus, sunny_plus) + dot(seen_minus, sunny_minus)) / seen_1);
    } else {
        light = (dot(look_4.plus, sunny_plus) + dot(look_4.minus, sunny_minus));
    }
    float _e41 = light;
    return _e41;
}

float3 lit_(float3 colour, float light_1)
{
    float _e5 = camera.sun.w;
    float _e9 = camera.daylight.y;
    float _e14 = camera.sun.w;
    float _e20 = camera.daylight.x;
    return (colour * ((_e5 * _e9) + (((1.0 - _e14) * _e20) * light_1)));
}

float3 sunlit(float3 colour_1, float light_2)
{
    float _e5 = camera.sun.w;
    float _e12 = camera.daylight.x;
    return (((colour_1 * (1.0 - _e5)) * _e12) * light_2);
}

float3 sky_tone(float3 day, float3 d_4)
{
    float3 colour_2 = (float3)0;

    float _e7 = camera.daylight.w;
    float3 night = ((day * NIGHT_SKY) * _e7);
    float _e12 = camera.daylight.z;
    colour_2 = lerp(night, day, _e12);
    float _e18 = camera.sun.z;
    float height = (asin(clamp(_e18, -1.0, 1.0)) * 57.29578);
    float dusk = exp(-(pow(((height + 2.0) / 4.0), 2.0)));
    float4 _e35 = camera.sun;
    float3 flat_sun = normalize((float3(_e35.xy, 0.0) + float3(1e-6, 0.0, 0.0)));
    float3 flat_d = normalize((float3(d_4.xy, 0.0) + float3(1e-6, 0.0, 0.0)));
    float towards = pow(max(dot(flat_d, flat_sun), 0.0), 3.0);
    float low_1 = pow((1.0 - clamp(abs(d_4.z), 0.0, 1.0)), 6.0);
    float3 _e68 = colour_2;
    float _e73 = camera.daylight.z;
    colour_2 = lerp(_e68, (DUSK * max(_e73, 0.15)), (((dusk * towards) * low_1) * 0.6));
    float3 _e82 = colour_2;
    return _e82;
}

float3 sky(float3 d_5)
{
    float up_1 = clamp(d_5.z, -0.2, 1.0);
    const float3 _e12 = sky_tone(lerp(HORIZON, ZENITH, pow(max(up_1, 0.0), 0.6)), d_5);
    return _e12;
}

Seen ConstructSeen(float3 arg0, float arg1, float arg2, int arg3, float3 arg4, float3 arg5, float arg6, float3 arg7, float arg8, float arg9) {
    Seen ret = (Seen)0;
    ret.colour = arg0;
    ret.distance_ = arg1;
    ret.steps = arg2;
    ret.level = arg3;
    ret.sunny = arg4;
    ret.at = arg5;
    ret.wide = arg6;
    ret.skyish = arg7;
    ret.touched = arg8;
    ret.sky_share = arg9;
    return ret;
}

Row ZeroValueRow() {
    return (Row)0;
}

Look ZeroValueLook() {
    return (Look)0;
}

Seen march(float3 o_3, float3 d_6, float phase, float begin, float end)
{
    float3 colour_3 = (0.0).xxx;
    float3 sunny = (0.0).xxx;
    float through = 1.0;
    float first = (float)0;
    float touched = (float)0;
    int steps = int(0);
    float t_1 = (float)0;
    bool in_air = true;
    float last_air = (float)0;
    bool walking = false;
    float walk_to = 0.0;
    int first_level = int(-1);
    Look before = (Look)0;
    float before_t = 0.0;
    bool chained = false;
    bool floored = false;
    bool bottom = false;
    float3 bottom_here = (0.0).xxx;
    float3 bottom_sun = (0.0).xxx;
    float bottom_t = 0.0;
    int bottom_level = int(-1);
    Place place_4 = ConstructPlace((int(0)).xx, false, false, 0u);
    Kept kept = (Kept)0;
    Kept kept_next = (Kept)0;
    Row row_own = (Row)0;
    Row row_up = (Row)0;
    Row row_far = (Row)0;
    int lv = FIRST;
    float lv_from = 1.0;
    float lv_to = 0.0;
    float lv_hand = 0.0;
    int n_1 = int(0);
    bool local_28 = (bool)0;
    bool local_29 = (bool)0;
    bool local_30 = (bool)0;
    bool local_31 = (bool)0;
    int level_12 = (int)0;
    bool skipped = (bool)0;
    int k_7 = (int)0;
    bool local_32 = (bool)0;
    bool local_33 = (bool)0;
    Row row_3 = (Row)0;
    float next = (float)0;
    float over = (float)0;
    bool local_34 = (bool)0;
    bool local_35 = (bool)0;
    float mixed = (float)0;
    Look look_5 = (Look)0;
    bool local_36 = (bool)0;
    bool local_37 = (bool)0;
    bool local_38 = (bool)0;
    float dt = (float)0;
    bool local_39 = (bool)0;
    bool local_40 = (bool)0;
    bool local_41 = (bool)0;
    bool local_42 = (bool)0;
    bool local_43 = (bool)0;
    float wide = 0.0;

    float _e16 = camera.reach.x;
    first = _e16;
    float _e21 = camera.reach.x;
    touched = _e21;
    t_1 = (max(begin, 0.5) + phase);
    float _e31 = t_1;
    last_air = _e31;
    const float _e44 = world_top();
    float ceiling = (_e44 + 32.0);
    float surface_step = (0.25 / clamp(abs(d_6.z), 0.25, 1.0));
    float4 _e56 = camera.water;
    float3 water_colour = _e56.xyz;
    const Kept _e79 = kept_none();
    kept = _e79;
    const Kept _e81 = kept_none();
    kept_next = _e81;
    const Row _e83 = row_none();
    row_own = _e83;
    const Row _e85 = row_none();
    row_up = _e85;
    const Row _e87 = row_none();
    row_far = _e87;
    float _e100 = camera.reach.x;
    float last = min(_e100, end);
    float _e105 = camera.pyramid.z;
    int most = naga_f2i32(_e105);
    uint2 loop_bound_6 = uint2(4294967295u, 4294967295u);
    bool loop_init_4 = true;
    while(true) {
        if (all(loop_bound_6 == uint2(0u, 0u))) { break; }
        loop_bound_6 -= uint2(loop_bound_6.y == 0u, 1u);
        if (!loop_init_4) {
            int _e710 = n_1;
            n_1 = asint(asuint(_e710) + asuint(int(1)));
        }
        loop_init_4 = false;
        int _e109 = n_1;
        if ((_e109 < most)) {
        } else {
            break;
        }
        {
            float _e111 = t_1;
            if (!((_e111 > last))) {
                float _e116 = through;
                local_28 = (_e116 < 0.01);
            } else {
                local_28 = true;
            }
            bool _e120 = local_28;
            if (_e120) {
                break;
            }
            int _e121 = n_1;
            steps = asint(asuint(_e121) + asuint(int(1)));
            float _e124 = t_1;
            float3 p_10 = (o_3 + (d_6 * _e124));
            if ((p_10.z > ceiling)) {
                local_29 = (d_6.z >= 0.0);
            } else {
                local_29 = false;
            }
            bool _e135 = local_29;
            if (_e135) {
                break;
            }
            if ((p_10.z < -8.0)) {
                local_30 = (d_6.z <= 0.0);
            } else {
                local_30 = false;
            }
            bool _e145 = local_30;
            if (_e145) {
                floored = true;
                break;
            }
            float _e147 = t_1;
            float _e148 = lv_from;
            if (!((_e147 < _e148))) {
                float _e153 = t_1;
                float _e154 = lv_to;
                local_31 = (_e153 >= _e154);
            } else {
                local_31 = true;
            }
            bool _e157 = local_31;
            if (_e157) {
                float _e158 = t_1;
                const int _e159 = level_at(_e158);
                lv = _e159;
                float _e160 = t_1;
                float _e164 = camera.view.z;
                float k_10 = floor(log2(max((_e160 * _e164), 1.0)));
                float _e174 = camera.view.z;
                lv_from = ((k_10 < 0.5) ? 0.0 : ((exp2(k_10) / _e174) * 1.00001));
                float _e188 = camera.view.z;
                lv_to = ((exp2((k_10 + 1.0)) / _e188) * 0.99999);
                int _e192 = lv;
                float _e197 = camera.reach.y;
                float _e202 = camera.fade.z;
                float _e208 = camera.view.z;
                lv_hand = ((exp2(((float(_e192) - _e197) + _e202)) / _e208) * 0.99999);
            }
            int _e212 = lv;
            level_12 = _e212;
            int _e214 = level_12;
            const int _e215 = ring_level(_e214, p_10);
            level_12 = _e215;
            keep_place(place_4, p_10);
            int _e216 = level_12;
            if ((_e216 > REGION)) {
                int _e220 = level_12;
                float2 c_2 = floor((p_10.xy / (float((int(1) << uint(_e220)))).xx));
                int _e228 = level_12;
                const uint _e233 = entry_at(_e228, naga_f2i32(c_2.x), naga_f2i32(c_2.y));
                if ((_e233 == NOT_SUMMED)) {
                    level_12 = REGION;
                }
            }
            int _e237 = level_12;
            if ((_e237 <= REGION)) {
                bool _e241 = place_4.summed;
                if (!(_e241)) {
                    int2 _e246 = place_4.region;
                    const float _e249 = leave2_(o_3.xy, d_6.xy, float2(_e246), 1024.0);
                    float _e250 = t_1;
                    t_1 = (max(_e249, _e250) + 0.01);
                    float _e254 = t_1;
                    last_air = _e254;
                    walking = false;
                    chained = false;
                    continue;
                }
            }
            skipped = false;
            int _e259 = level_12;
            float fine = float((int(1) << uint(_e259)));
            k_7 = int(0);
            uint2 loop_bound_7 = uint2(4294967295u, 4294967295u);
            bool loop_init_5 = true;
            while(true) {
                if (all(loop_bound_7 == uint2(0u, 0u))) { break; }
                loop_bound_7 -= uint2(loop_bound_7.y == 0u, 1u);
                if (!loop_init_5) {
                    int _e352 = k_7;
                    k_7 = asint(asuint(_e352) + asuint(int(1)));
                }
                loop_init_5 = false;
                int _e266 = k_7;
                if ((_e266 < int(3))) {
                    bool _e271 = in_air;
                    local_32 = _e271;
                } else {
                    local_32 = false;
                }
                bool _e273 = local_32;
                if (_e273) {
                    bool _e276 = walking;
                    local_33 = !(_e276);
                } else {
                    local_33 = false;
                }
                bool _e279 = local_33;
                if (_e279) {
                } else {
                    break;
                }
                {
                    int _e280 = level_12;
                    int _e283 = k_7;
                    int up_2 = min(asint(asuint(asint(asuint(_e280) + asuint(int(4)))) - asuint(asint(asuint(int(2)) * asuint(_e283)))), TOP);
                    float up_size = float((int(1) << uint(up_2)));
                    float2 cc = floor((p_10.xy / (up_size).xx));
                    row_3 = ZeroValueRow();
                    int _e299 = k_7;
                    if ((_e299 == int(0))) {
                        keep_row(row_far, up_2);
                        Row _e302 = row_far;
                        row_3 = _e302;
                    } else {
                        int _e303 = k_7;
                        if ((_e303 == int(1))) {
                            keep_row(row_up, up_2);
                            Row _e306 = row_up;
                            row_3 = _e306;
                        } else {
                            keep_row(row_own, up_2);
                            Row _e307 = row_own;
                            row_3 = _e307;
                        }
                    }
                    Row _e308 = row_3;
                    const uint _e313 = top_in(_e308, naga_f2i32(cc.x), naga_f2i32(cc.y));
                    float top = ((floor(((float(_e313) - 1.0) / fine)) + 2.0) * fine);
                    if ((p_10.z <= top)) {
                        continue;
                    }
                    const float _e326 = leave2_(o_3.xy, d_6.xy, cc, up_size);
                    next = _e326;
                    if ((d_6.z < 0.0)) {
                        float _e331 = next;
                        float _e332 = t_1;
                        next = min(_e331, (_e332 + ((top - p_10.z) / d_6.z)));
                    }
                    float _e339 = next;
                    float _e340 = t_1;
                    if ((_e339 > (_e340 + 0.01))) {
                        float _e344 = next;
                        t_1 = (_e344 + 0.01);
                        float _e347 = t_1;
                        last_air = _e347;
                        walking = false;
                        chained = false;
                        skipped = true;
                        break;
                    }
                }
            }
            bool _e354 = skipped;
            if (_e354) {
                continue;
            }
            over = 0.0;
            int _e357 = level_12;
            if ((_e357 < TOP)) {
                float _e362 = t_1;
                float _e363 = lv_hand;
                if (!((_e362 > _e363))) {
                    int _e368 = level_12;
                    int _e369 = lv;
                    local_35 = (_e368 != _e369);
                } else {
                    local_35 = true;
                }
                bool _e372 = local_35;
                local_34 = _e372;
            } else {
                local_34 = false;
            }
            bool _e374 = local_34;
            if (_e374) {
                float _e375 = t_1;
                float _e379 = camera.view.z;
                float _e387 = camera.reach.y;
                int _e389 = level_12;
                over = ((log2(max((_e375 * _e379), 1.0)) + _e387) - float(_e389));
            }
            float _e392 = over;
            float _e396 = camera.fade.z;
            float _e401 = camera.fade.z;
            mixed = ((_e392 - _e396) / (1.0 - _e401));
            int _e406 = level_12;
            if ((_e406 < FIRST)) {
                float _e409 = mixed;
                int _e410 = level_12;
                const float _e411 = in_ring(_e410, p_10);
                int _e413 = level_12;
                mixed = max(_e409, (1.0 - (_e411 / (RING_FADE * float((int(1) << uint(_e413)))))));
            }
            look_5 = ZeroValueLook();
            float _e425 = mixed;
            if ((_e425 > 0.0)) {
                Place _e428 = place_4;
                int _e429 = level_12;
                const Spot _e430 = spot_kept(kept, _e428, _e429, p_10);
                Place _e431 = place_4;
                int _e432 = level_12;
                const Spot _e435 = spot_kept(kept_next, _e431, asint(asuint(_e432) + asuint(int(1))), p_10);
                if ((_e430.kind == int(2))) {
                    local_36 = (_e435.kind == int(2));
                } else {
                    local_36 = false;
                }
                bool _e445 = local_36;
                if (_e445) {
                    const Fetched _e447 = fetch_together(_e430.at);
                    const Fetched _e449 = fetch_together(_e435.at);
                    const Look _e450 = look_of(_e447);
                    look_5 = _e450;
                    Look _e451 = look_5;
                    const Look _e452 = look_of(_e449);
                    float _e453 = mixed;
                    const Look _e454 = mix_looks(_e451, _e452, _e453);
                    look_5 = _e454;
                } else {
                    const Look _e455 = look_at(_e430, p_10);
                    look_5 = _e455;
                    bool _e457 = look_5.known;
                    if (_e457) {
                        const Look _e458 = look_at(_e435, p_10);
                        if (_e458.known) {
                            Look _e460 = look_5;
                            float _e461 = mixed;
                            const Look _e462 = mix_looks(_e460, _e458, _e461);
                            look_5 = _e462;
                        }
                    }
                }
            } else {
                Place _e463 = place_4;
                int _e464 = level_12;
                const Look _e465 = far_blend_kept(kept, _e463, _e464, p_10);
                look_5 = _e465;
            }
            int _e466 = level_12;
            float size_3 = float((int(1) << uint(_e466)));
            float _e472 = look_5.solid;
            float _e474 = look_5.water;
            bool _e477 = look_5.known;
            float thick = (_e477 ? max(_e472, _e474) : 0.0);
            if ((thick > 0.0)) {
                bool _e484 = walking;
                local_37 = !(_e484);
            } else {
                local_37 = false;
            }
            bool _e487 = local_37;
            if (_e487) {
                float _e490 = t_1;
                float _e491 = last_air;
                local_38 = ((_e490 - _e491) > (size_3 * 0.26));
            } else {
                local_38 = false;
            }
            bool _e497 = local_38;
            if (_e497) {
                float _e498 = t_1;
                walk_to = _e498;
                float _e499 = last_air;
                t_1 = (_e499 + ((phase * size_3) * 0.25));
                walking = true;
                in_air = false;
                chained = false;
                continue;
            }
            dt = (size_3 * 2.0);
            in_air = true;
            if ((thick > 0.0)) {
                in_air = false;
                walking = true;
                float _e518 = camera.position.w;
                dt = ((thick > 0.02) ? (size_3 * surface_step) : (size_3 * _e518));
            } else {
                bool _e524 = walking;
                if (_e524) {
                    float _e527 = t_1;
                    float _e528 = walk_to;
                    local_39 = (_e527 < _e528);
                } else {
                    local_39 = false;
                }
                bool _e531 = local_39;
                if (_e531) {
                    in_air = false;
                    dt = (size_3 * surface_step);
                } else {
                    float _e534 = t_1;
                    last_air = _e534;
                    walking = false;
                    if ((d_6.z < 0.0)) {
                        float _e539 = dt;
                        dt = min(_e539, max((p_10.z / -(d_6.z)), (size_3 * 0.25)));
                    }
                }
            }
            bool _e549 = look_5.known;
            if (_e549) {
                bool _e552 = chained;
                local_40 = _e552;
            } else {
                local_40 = false;
            }
            bool _e554 = local_40;
            if (_e554) {
                bool _e558 = before.known;
                local_41 = _e558;
            } else {
                local_41 = false;
            }
            bool _e560 = local_41;
            if (_e560) {
                float _e561 = t_1;
                float _e562 = before_t;
                float length_ = (_e561 - _e562);
                float _e565 = before.solid;
                float _e567 = look_5.solid;
                float solid_share = (0.5 * (_e565 + _e567));
                float _e572 = before.water;
                float _e574 = look_5.water;
                float water_share = (0.5 * (_e572 + _e574));
                float solid = (1.0 - pow((1.0 - min(solid_share, 0.999)), (length_ / size_3)));
                float _e589 = camera.water.w;
                float water = (1.0 - exp(((-(_e589) * water_share) * length_)));
                float covered = (1.0 - ((1.0 - solid) * (1.0 - water)));
                if ((covered > 0.0)) {
                    float _e605 = touched;
                    float _e606 = before_t;
                    touched = min(_e605, _e606);
                    Look _e608 = before;
                    const float _e609 = faces_light(_e608, d_6);
                    Look _e610 = look_5;
                    const float _e611 = faces_light(_e610, d_6);
                    float _e613 = before.solid;
                    float _e615 = look_5.solid;
                    float solids = max((_e613 + _e615), 1e-6);
                    float3 _e620 = before.colour;
                    const float3 _e621 = lit_(_e620, _e609);
                    float _e623 = before.solid;
                    float3 _e626 = look_5.colour;
                    const float3 _e627 = lit_(_e626, _e611);
                    float _e629 = look_5.solid;
                    float3 ground = (((_e621 * _e623) + (_e627 * _e629)) / (solids).xxx);
                    float3 _e635 = before.colour;
                    const float3 _e636 = sunlit(_e635, _e609);
                    float _e638 = before.solid;
                    float3 _e641 = look_5.colour;
                    const float3 _e642 = sunlit(_e641, _e611);
                    float _e644 = look_5.solid;
                    float3 ground_sun = (((_e636 * _e638) + (_e642 * _e644)) / (solids).xxx);
                    float both = max((solid + water), 1e-6);
                    const float3 _e654 = lit_(water_colour, 1.0);
                    float3 here_1 = (((ground * solid) + (_e654 * water)) / (both).xxx);
                    const float3 _e661 = sunlit(water_colour, 1.0);
                    float3 here_sun = (((ground_sun * solid) + (_e661 * water)) / (both).xxx);
                    float3 _e666 = colour_3;
                    float _e667 = through;
                    colour_3 = (_e666 + ((_e667 * covered) * here_1));
                    float3 _e671 = sunny;
                    float _e672 = through;
                    sunny = (_e671 + ((_e672 * covered) * here_sun));
                    if ((p_10.z <= (size_3 * 1.5))) {
                        bottom = true;
                        bottom_here = here_1;
                        bottom_sun = here_sun;
                        float _e681 = t_1;
                        bottom_t = _e681;
                        int _e682 = level_12;
                        bottom_level = _e682;
                    }
                    float _e683 = through;
                    if ((_e683 > 0.5)) {
                        float _e688 = through;
                        local_42 = ((_e688 * (1.0 - covered)) <= 0.5);
                    } else {
                        local_42 = false;
                    }
                    bool _e695 = local_42;
                    if (_e695) {
                        float _e696 = t_1;
                        first = _e696;
                        int _e697 = level_12;
                        first_level = _e697;
                    }
                    float _e698 = through;
                    through = (_e698 * (1.0 - covered));
                }
            }
            Look _e702 = look_5;
            before = _e702;
            float _e703 = t_1;
            before_t = _e703;
            bool _e705 = look_5.known;
            chained = _e705;
            float _e706 = t_1;
            float _e707 = dt;
            t_1 = (_e706 + _e707);
        }
    }
    bool _e712 = floored;
    if (_e712) {
        bool _e715 = bottom;
        local_43 = _e715;
    } else {
        local_43 = false;
    }
    bool _e717 = local_43;
    if (_e717) {
        float3 _e718 = colour_3;
        float _e719 = through;
        float3 _e720 = bottom_here;
        colour_3 = (_e718 + (_e719 * _e720));
        float3 _e723 = sunny;
        float _e724 = through;
        float3 _e725 = bottom_sun;
        sunny = (_e723 + (_e724 * _e725));
        through = 0.0;
        float _e729 = first;
        float _e733 = camera.reach.x;
        if ((_e729 >= _e733)) {
            float _e735 = bottom_t;
            first = _e735;
            int _e736 = bottom_level;
            first_level = _e736;
        }
    }
    float3 gathered = colour_3;
    float3 _e738 = colour_3;
    float _e739 = through;
    const float3 _e740 = sky(d_6);
    colour_3 = (_e738 + (_e739 * _e740));
    float _e745 = first;
    float _e749 = camera.reach.x;
    if ((_e745 < _e749)) {
        int _e751 = first_level;
        int _e758 = first_level;
        float read = ((_e758 >= int(0)) ? float((int(1) << uint(max(_e751, int(0))))) : 1.0);
        float _e763 = first;
        float _e767 = camera.view.z;
        wide = max((_e763 * _e767), read);
    }
    float3 _e770 = colour_3;
    float _e771 = first;
    int _e772 = steps;
    int _e774 = first_level;
    float3 _e775 = sunny;
    float _e776 = first;
    float _e779 = wide;
    float3 _e780 = sunny;
    float _e782 = touched;
    float _e783 = through;
    const Seen seen_2 = ConstructSeen(_e770, _e771, float(_e772), _e774, _e775, (o_3 + (d_6 * _e776)), _e779, (gathered - _e780), _e782, _e783);
    return seen_2;
}

float scatter(uint2 at_4)
{
    uint h = (uint)0;

    h = ((at_4.x * 2654435761u) ^ (at_4.y * 2246822519u));
    uint _e9 = h;
    uint _e10 = h;
    h = ((_e9 ^ (_e10 >> 15u)) * 739982445u);
    uint _e16 = h;
    uint _e17 = h;
    h = (_e16 ^ (_e17 >> 12u));
    uint _e21 = h;
    return (float((_e21 & 65535u)) / 65536.0);
}

float weave(uint2 at_5, uint bits)
{
    uint v_3 = 0u;
    uint b_1 = 0u;

    uint x_5 = (at_5.x ^ at_5.y);
    uint2 loop_bound_8 = uint2(4294967295u, 4294967295u);
    bool loop_init_6 = true;
    while(true) {
        if (all(loop_bound_8 == uint2(0u, 0u))) { break; }
        loop_bound_8 -= uint2(loop_bound_8.y == 0u, 1u);
        if (!loop_init_6) {
            uint _e28 = b_1;
            b_1 = (_e28 + 1u);
        }
        loop_init_6 = false;
        uint _e9 = b_1;
        if ((_e9 < bits)) {
        } else {
            break;
        }
        {
            uint _e11 = v_3;
            uint _e14 = b_1;
            uint _e22 = b_1;
            v_3 = (((_e11 << 2u) | (((x_5 >> _e14) & 1u) << 1u)) | ((at_5.y >> _e22) & 1u));
        }
    }
    uint _e30 = v_3;
    return ((float(_e30) + 0.5) / float((1u << (2u * bits))));
}

uint2 naga_mod(uint2 lhs, uint2 rhs) {
    return lhs % (rhs == 0u ? 1u : rhs);
}

float blue(uint2 at_6)
{
    uint2 side = NagaDimensions2D(t_blue);
    float4 _e7 = t_blue.Load(int3(int2(naga_mod(at_6, side)), int(0)));
    return (((_e7.x * 255.0) + 0.5) / 256.0);
}

float ray_start(uint2 at_7)
{
    float phase_1 = (float)0;
    bool local_44 = (bool)0;

    float _e4 = camera.ray_start.x;
    uint pattern = naga_f2u32(_e4);
    const float _e8 = scatter((at_7 * 2u));
    phase_1 = _e8;
    if (!((pattern == 1u))) {
        local_44 = (pattern == 3u);
    } else {
        local_44 = true;
    }
    bool _e18 = local_44;
    if (_e18) {
        float _e22 = camera.ray_start.y;
        const float _e24 = weave(at_7, naga_f2u32(_e22));
        phase_1 = _e24;
        if ((pattern == 3u)) {
            float _e27 = phase_1;
            float _e31 = camera.ray_start.z;
            const float _e32 = blue(at_7);
            phase_1 = frac((_e27 + (_e31 * _e32)));
        }
    } else {
        if ((pattern == 2u)) {
            const float _e38 = blue(at_7);
            phase_1 = _e38;
        }
    }
    float _e39 = phase_1;
    float _e43 = camera.ray_start.w;
    return (_e39 * _e43);
}

float worth_of(float near, int level_13)
{
    bool local_45 = (bool)0;

    float _e5 = camera.layer.y;
    if ((_e5 < 0.5)) {
        return 0.0;
    }
    float _e12 = camera.layer.z;
    if ((_e12 < 0.5)) {
        return 1.0;
    }
    float _e19 = camera.reach.x;
    if (!((near >= _e19))) {
        local_45 = (level_13 < int(0));
    } else {
        local_45 = true;
    }
    bool _e27 = local_45;
    if (_e27) {
        return 0.0;
    }
    float cell = float((int(1) << uint(level_13)));
    float _e36 = camera.view.z;
    float _e41 = camera.pyramid.y;
    float ratio = (((near * _e36) * _e41) / cell);
    float _e47 = camera.fade.x;
    float _e54 = camera.fade.y;
    return clamp((((ratio / _e47) - 1.0) / max((exp2(_e54) - 1.0), 0.001)), 0.0, 1.0);
}

float2 light_of(int level_14, float3 p_8)
{
    float size_4 = float((int(1) << uint(level_14)));
    const int _e7 = table(level_14, int(1));
    const int _e10 = table(level_14, int(2));
    float2 least = float2(float(_e7), float(_e10));
    const int _e14 = table(level_14, int(3));
    float across = float(_e14);
    const int _e17 = table(level_14, int(11));
    float2 at_8 = float2(float((_e17 & int(65535))), float((_e17 >> 16u)));
    float2 c_3 = clamp(((p_8.xy / (size_4).xx) - least), (0.5).xx, ((across - 0.5)).xx);
    float2 uv = ((c_3 + at_8) / float2(NagaDimensions2D(t_shadow)));
    float4 _e43 = t_shadow.SampleLevel(soft, uv, 0.0);
    float shadow = _e43.x;
    float4 _e48 = t_sky.SampleLevel(soft, uv, 0.0);
    float sky_1 = _e48.x;
    return float2(clamp((((p_8.z - shadow) / size_4) + 1.5), 0.0, 1.0), sky_1);
}

float2 cell_light(float3 p_9, float footprint)
{
    float f_2 = clamp(log2(max(footprint, 1.0)), 5.0, 16.0);
    int level_16 = naga_f2i32(floor(f_2));
    const float2 _e10 = light_of(level_16, p_9);
    if ((level_16 >= TOP)) {
        return _e10;
    }
    const float2 _e15 = light_of(asint(asuint(level_16) + asuint(int(1))), p_9);
    return lerp(_e10, _e15, (f_2 - floor(f_2)));
}

float3 shaded(Seen seen)
{
    float3 colour_4 = (float3)0;
    float sun = 1.0;
    float sky_open = 1.0;

    if ((seen.wide <= 0.0)) {
        return seen.colour;
    }
    colour_4 = seen.colour;
    float _e14 = camera.view.w;
    if ((_e14 > 0.5)) {
        const float2 _e19 = cell_light(seen.at, seen.wide);
        float _e24 = camera.follow.y;
        sun = ((_e24 > 0.5) ? _e19.x : 1.0);
        sky_open = _e19.y;
    }
    float3 _e30 = colour_4;
    float _e32 = sun;
    colour_4 = (_e30 - (seen.sunny * (1.0 - _e32)));
    float _e37 = sky_open;
    float _e41 = camera.sky.y;
    float _e47 = camera.sky.x;
    float sky_2 = ((_e47 > 0.5) ? lerp(1.0, _e37, _e41) : 1.0);
    float _e58 = camera.sky.z;
    float3 tint = lerp((1.0).xxx, SKY_TINT, _e58);
    float3 _e60 = colour_4;
    return (_e60 + (seen.skyish * ((sky_2 * tint) - (1.0).xxx)));
}

float3 fogged(float3 colour_5, float3 d_7, float t_2)
{
    float along = (float)0;

    float _e6 = camera.fog.x;
    if ((_e6 <= 0.0)) {
        return colour_5;
    }
    float thins = camera.fog.y;
    float _e16 = camera.fog.x;
    float _e21 = camera.position.z;
    float _e25 = camera.fog.z;
    float at_eye = (_e16 * exp(clamp((-(thins) * (_e21 - _e25)), -80.0, 80.0)));
    float climb = (thins * d_7.z);
    along = t_2;
    if ((abs((climb * t_2)) > 0.0001)) {
        along = ((1.0 - exp(-(clamp((climb * t_2), -80.0, 80.0)))) / climb);
    }
    float _e50 = along;
    float amount = (1.0 - exp((-(at_eye) * _e50)));
    float4 _e57 = camera.sun;
    float _e67 = camera.fog.w;
    float _e72 = camera.daylight.x;
    float sunward = ((pow(max(dot(d_7, _e57.xyz), 0.0), 8.0) * _e67) * _e72);
    const float3 _e77 = sky_tone(lerp(HORIZON, FOG_WARM, sunward), d_7);
    return lerp(colour_5, _e77, amount);
}

float3 to_srgb(float3 c_1)
{
    float3 low_2 = (c_1 * 12.92);
    float3 high = ((1.055 * pow(c_1, (0.41666666).xxx)) - (0.055).xxx);
    return ((c_1 <= (0.0031308).xxx) ? low_2 : high);
}

int naga_mod(int lhs, int rhs) {
    int divisor = ((lhs == int(-2147483647 - 1) & rhs == -1) | (rhs == 0)) ? 1 : rhs;
    return lhs - (lhs / divisor) * divisor;
}

int naga_div(int lhs, int rhs) {
    return lhs / (((lhs == int(-2147483647 - 1) & rhs == -1) | (rhs == 0)) ? 1 : rhs);
}

typedef float3 ret_Constructarray6_float3_[6];
ret_Constructarray6_float3_ Constructarray6_float3_(float3 arg0, float3 arg1, float3 arg2, float3 arg3, float3 arg4, float3 arg5) {
    float3 ret[6] = { arg0, arg1, arg2, arg3, arg4, arg5 };
    return ret;
}

uint2 naga_f2u32(float2 value) {
    return uint2(clamp(value, 0.0, 4294967000.0));
}

void march_pixel(uint2 id_1)
{
    bool local_46 = (bool)0;
    float shown = 1.0;
    float begin_1 = 0.5;
    bool local_47 = (bool)0;
    float nearest = (float)0;
    int k_8 = int(0);
    float end_1 = (float)0;
    float3 here = (float3)0;
    float3 colour_6 = (float3)0;
    bool local_48 = (bool)0;
    bool local_49 = (bool)0;

    float4 _e3 = camera.view;
    uint2 size_5 = naga_f2u32(_e3.xy);
    if (!((id_1.x >= size_5.x))) {
        local_46 = (id_1.y >= size_5.y);
    } else {
        local_46 = true;
    }
    bool _e16 = local_46;
    if (_e16) {
        return;
    }
    float _e27 = camera.pyramid.x;
    float _e33 = camera.layer.x;
    uint2 parent = min(naga_f2u32(((float2(id_1.xy) + (0.5).xx) * _e27)), uint2((naga_f2u32(max(_e33, 1.0)) - 1u), 65535u));
    float _e45 = camera.layer.x;
    if ((_e45 > 0.0)) {
        float _e52 = camera.layer.x;
        const uint _e57 = get_worth(((parent.y * naga_f2u32(_e52)) + parent.x));
        uint worth = (_e57 & 255u);
        shown = (float(worth) / 255.0);
        if ((worth == 0u)) {
            put_picture(int2(id_1.xy), (0.0).xxxx);
            float _e74 = camera.reach.x;
            put_far(int2(id_1.xy), float4(_e74, 0.0, 0.0, 0.0));
            float _e82 = camera.layer.y;
            if ((_e82 > 0.5)) {
                put_worth(((id_1.y * size_5.x) + id_1.x), 0u);
            }
            return;
        }
    }
    float _e96 = camera.tiles.z;
    if ((_e96 > 0.0)) {
        float _e104 = camera.tiles.z;
        uint2 tile = naga_f2u32((float2(id_1.xy) / (_e104).xx));
        float _e113 = camera.tiles.x;
        float _e119 = asfloat(starts.Load(((tile.y * naga_f2u32(_e113)) + tile.x)*4));
        begin_1 = _e119;
    }
    float _e123 = camera.layer.x;
    if ((_e123 > 0.0)) {
        float _e131 = camera.follow.x;
        local_47 = (_e131 > 0.5);
    } else {
        local_47 = false;
    }
    bool _e135 = local_47;
    if (_e135) {
        int2 above = int2(parent);
        const int2 _e137 = far_above_edge();
        float _e141 = camera.reach.x;
        nearest = _e141;
        uint2 loop_bound_9 = uint2(4294967295u, 4294967295u);
        bool loop_init_7 = true;
        while(true) {
            if (all(loop_bound_9 == uint2(0u, 0u))) { break; }
            loop_bound_9 -= uint2(loop_bound_9.y == 0u, 1u);
            if (!loop_init_7) {
                int _e167 = k_8;
                k_8 = asint(asuint(_e167) + asuint(int(1)));
            }
            loop_init_7 = false;
            int _e145 = k_8;
            if ((_e145 < int(9))) {
            } else {
                break;
            }
            {
                int _e148 = k_8;
                int _e153 = k_8;
                int2 at_9 = clamp(asint(asuint(above) + asuint(int2(asint(asuint(naga_mod(_e148, int(3))) - asuint(int(1))), asint(asuint(naga_div(_e153, int(3))) - asuint(int(1)))))), (int(0)).xx, _e137);
                float _e163 = nearest;
                const float _e164 = get_far_above(at_9);
                nearest = min(_e163, _e164);
            }
        }
        float _e169 = begin_1;
        float _e170 = nearest;
        float _e173 = nearest;
        float _e177 = camera.view.z;
        begin_1 = max(_e169, ((_e170 - 2.0) - ((_e173 * _e177) * 4.0)));
    }
    float _e183 = begin_1;
    float _e187 = camera.sky.w;
    begin_1 = max(_e183, _e187);
    float2 uv_1 = ((float2(id_1.xy) + (0.5).xx) / float2(size_5));
    float4 _e198 = camera.forward;
    float4 _e207 = camera.right;
    float4 _e218 = camera.up;
    float3 d_8 = normalize(((_e198.xyz + (((uv_1.x * 2.0) - 1.0) * _e207.xyz)) + ((1.0 - (uv_1.y * 2.0)) * _e218.xyz)));
    float _e226 = camera.reach.x;
    end_1 = _e226;
    float _e231 = camera.follow.w;
    if ((_e231 > 0.5)) {
        float4 _e236 = camera.position;
        const float _e238 = map_end(_e236.xyz, d_8);
        end_1 = _e238;
    }
    float4 _e241 = camera.position;
    const float _e244 = ray_start(id_1.xy);
    float _e245 = begin_1;
    float _e246 = end_1;
    const Seen _e247 = march(_e241.xyz, d_8, _e244, _e245, _e246);
    const float _e250 = worth_of(_e247.distance_, _e247.level);
    bool hidden = (round((_e250 * 255.0)) >= 255.0);
    here = _e247.colour;
    if (!(hidden)) {
        const float3 _e259 = shaded(_e247);
        here = _e259;
    }
    float _e266 = camera.reach.x;
    float met = ((_e247.distance_ >= _e266) ? _e247.touched : _e247.distance_);
    float _e272 = camera.reach.x;
    if ((met < _e272)) {
        const float3 _e275 = sky(d_8);
        float3 behind = (_e247.sky_share * _e275);
        float covered_1 = max((1.0 - _e247.sky_share), 0.0001);
        float3 _e282 = here;
        const float3 _e286 = fogged(((_e282 - behind) / (covered_1).xxx), d_8, met);
        float _e290 = camera.fade.w;
        float shown_2 = pow(covered_1, (1.0 + ((_e290 * _e247.sky_share) * _e247.sky_share)));
        const float3 _e298 = sky(d_8);
        here = ((_e298 * (1.0 - shown_2)) + (_e286 * shown_2));
    }
    float _e307 = camera.layer.y;
    if ((_e307 > 0.5)) {
        const float _e317 = worth_of(_e247.distance_, _e247.level);
        put_worth(((id_1.y * size_5.x) + id_1.x), naga_f2u32(round((_e317 * 255.0))));
    }
    float3 _e322 = here;
    colour_6 = _e322;
    float _e327 = camera.reach.z;
    if ((_e327 > 1.5)) {
        local_48 = (_e247.level >= int(0));
    } else {
        local_48 = false;
    }
    bool _e336 = local_48;
    if (_e336) {
        float3 tints[6] = Constructarray6_float3_(float3(1.0, 0.3, 0.3), float3(0.3, 1.0, 0.3), float3(0.3, 0.4, 1.0), float3(1.0, 1.0, 0.3), float3(1.0, 0.3, 1.0), float3(0.3, 1.0, 1.0));
        float3 _e362 = colour_6;
        colour_6 = ((_e362 * 0.4) + (tints[min(uint(naga_mod(_e247.level, int(6))), 5u)] * 0.35));
    } else {
        float _e375 = camera.reach.z;
        if ((_e375 > 0.5)) {
            float _e383 = camera.reach.z;
            local_49 = (_e383 < 1.5);
        } else {
            local_49 = false;
        }
        bool _e387 = local_49;
        if (_e387) {
            float k_11 = clamp((_e247.steps / 256.0), 0.0, 1.0);
            colour_6 = ((k_11 < 0.5) ? lerp(float3(0.0, 0.0, 0.4), float3(0.0, 0.8, 0.2), (k_11 * 2.0)) : lerp(float3(0.0, 0.8, 0.2), float3(1.0, 0.0, 0.0), ((k_11 - 0.5) * 2.0)));
            float3 _e421 = colour_6;
            float3 _e422 = colour_6;
            colour_6 = (_e421 * _e422);
        }
    }
    float3 _e426 = colour_6;
    const float3 _e432 = to_srgb(clamp(_e426, (0.0).xxx, (1.0).xxx));
    float _e433 = shown;
    float _e435 = shown;
    put_picture(int2(id_1.xy), float4((_e432 * _e433), _e435));
    put_far(int2(id_1.xy), float4(_e247.touched, _e247.steps, 0.0, 0.0));
    return;
}

[numthreads(8, 8, 1)]
void march_pass(uint3 id : SV_DispatchThreadID)
{
    march_pixel(id.xy);
    return;
}

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

cbuffer cameras : register(b0) { Camera cameras[3]; }
ByteAddressBuffer levels : register(t0);
ByteAddressBuffer entries : register(t1);
ByteAddressBuffer index : register(t2);
ByteAddressBuffer pool : register(t3);
RWByteAddressBuffer starts : register(u2);
RWTexture2D<unorm float4> picture0_ : register(u0);
RWTexture2D<unorm float4> picture1_ : register(u4);
RWTexture2D<unorm float4> picture2_ : register(u5);
RWTexture2D<float4> how_far : register(u1);
ByteAddressBuffer worth_in : register(t4);
RWByteAddressBuffer worth_out : register(u3);
Texture2D<float4> far_prev : register(t5);
static uint layer_now = (uint)0;
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

uint naga_f2u32(float value) {
    return uint(clamp(value, 0.0, 4294967000.0));
}

uint base_pixels()
{
    float _e4 = cameras[0].view.x;
    float _e10 = cameras[0].view.y;
    return (naga_f2u32(_e4) * naga_f2u32(_e10));
}

void put_picture(int2 at_1, float4 v)
{
    uint _e3 = layer_now;
    switch(_e3) {
        case 0u: {
            picture0_[at_1] = v;
            return;
        }
        case 1u: {
            picture1_[at_1] = v;
            return;
        }
        default: {
            picture2_[at_1] = v;
            return;
        }
    }
}

int naga_f2i32(float value) {
    return int(clamp(value, -2147483600.0, 2147483500.0));
}

int far_left(uint layer)
{
    int left = int(0);

    if ((layer > 0u)) {
        int _e5 = left;
        float _e10 = cameras[0].view.x;
        left = asint(asuint(_e5) + asuint(naga_f2i32(_e10)));
    }
    if ((layer > 1u)) {
        int _e15 = left;
        float _e20 = cameras[1].view.x;
        left = asint(asuint(_e15) + asuint(naga_f2i32(_e20)));
    }
    int _e23 = left;
    return _e23;
}

void put_far(int2 at_2, float4 v_1)
{
    uint _e4 = layer_now;
    const int _e5 = far_left(_e4);
    how_far[asint(asuint(at_2) + asuint(int2(_e5, int(0))))] = v_1;
    return;
}

void put_worth(uint i, uint v_2)
{
    const uint _e4 = base_pixels();
    uint _e6 = layer_now;
    worth_out.Store((i + ((_e6 == 1u) ? _e4 : 0u))*4, asuint(v_2));
    return;
}

uint get_worth(uint i_1)
{
    const uint _e3 = base_pixels();
    uint _e5 = layer_now;
    uint _e11 = asuint(worth_in.Load((i_1 + ((_e5 == 2u) ? _e3 : 0u))*4));
    return _e11;
}

int2 naga_f2i32(float2 value) {
    return int2(clamp(value, -2147483600.0, 2147483500.0));
}

int2 far_above_edge()
{
    uint _e2 = layer_now;
    float4 _e9 = cameras[(max(_e2, 1u) - 1u)].view;
    return asint(asuint(naga_f2i32(_e9.xy)) - asuint((int(1)).xx));
}

float get_far_above(int2 at_3)
{
    uint _e3 = layer_now;
    const int _e8 = far_left((max(_e3, 1u) - 1u));
    float4 _e13 = far_prev.Load(int3(asint(asuint(at_3) + asuint(int2(_e8, int(0)))), int(0)));
    return _e13.x;
}

int table(int level, int k)
{
    int row_4 = ((level < FIRST) ? asint(asuint(int(12)) + asuint(asint(asuint(int(4)) - asuint(level)))) : asint(asuint(level) - asuint(FIRST)));
    int _e16 = asint(levels.Load(asint(asuint(asint(asuint(row_4) * asuint(TABLE))) + asuint(k))*4));
    return _e16;
}

int level_at(float t)
{
    uint _e3 = layer_now;
    float _e7 = cameras[_e3].view.z;
    float footprint_1 = max((t * _e7), 1.0);
    uint _e16 = layer_now;
    float _e20 = cameras[_e16].reach.y;
    int level_15 = asint(asuint(naga_f2i32(floor(log2(footprint_1)))) + asuint(naga_f2i32(_e20)));
    uint _e25 = layer_now;
    float _e29 = cameras[_e25].follow.z;
    return clamp(level_15, naga_f2i32(_e29), TOP);
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
        uint _e33 = layer_now;
        float _e37 = cameras[_e33].reach.x;
        return _e37;
    }
    const float _e40 = leaves(o_1.xy, d_1.xy, lo_1, hi_1);
    const int _e41 = level_at(_e40);
    float s = (2.0 * float((int(1) << uint(min(asint(asuint(_e41) + asuint(int(1))), TOP)))));
    const float _e58 = leaves(o_1.xy, d_1.xy, (lo_1 - (s).xx), (hi_1 + (s).xx));
    return _e58;
}

float world_top()
{
    uint _e2 = layer_now;
    float _e6 = cameras[_e2].reach.w;
    return min(max(_e6, 0.0), WORLD_HEIGHT);
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
    uint _e4 = layer_now;
    float4 box = cameras[_e4].rings[asint(asuint(int(4)) - asuint(level_1))];
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
    uint _e22 = layer_now;
    float _e26 = cameras[_e22].window.y;
    int n_2 = naga_f2i32(_e26);
    uint _e30 = layer_now;
    float4 _e33 = cameras[_e30].window;
    int2 rr = asint(asuint(r) - asuint(naga_f2i32(_e33.zw)));
    if ((rr.x >= int(0))) {
        local_6 = (rr.y >= int(0));
    } else {
        local_6 = false;
    }
    bool _e46 = local_6;
    if (_e46) {
        local_7 = (rr.x < n_2);
    } else {
        local_7 = false;
    }
    bool _e52 = local_7;
    if (_e52) {
        local_8 = (rr.y < n_2);
    } else {
        local_8 = false;
    }
    bool on_map = local_8;
    const int _e61 = table(TOP, int(4));
    const int _e64 = table(TOP, int(3));
    const int _e67 = table(TOP, int(3));
    int flags_at = asint(asuint(_e61) + asuint(asint(asuint(_e64) * asuint(_e67))));
    uint2 loop_bound_1 = uint2(4294967295u, 4294967295u);
    bool loop_init = true;
    while(true) {
        if (all(loop_bound_1 == uint2(0u, 0u))) { break; }
        loop_bound_1 -= uint2(loop_bound_1.y == 0u, 1u);
        if (!loop_init) {
            int _e109 = k_1;
            k_1 = asint(asuint(_e109) + asuint(int(1)));
        }
        loop_init = false;
        int _e72 = k_1;
        uint _e75 = layer_now;
        float _e79 = cameras[_e75].window.x;
        if ((_e72 < naga_f2i32(_e79))) {
        } else {
            break;
        }
        {
            if (!(!(on_map))) {
                int _e87 = k_1;
                uint _e97 = asuint(entries.Load(uint(asint(asuint(asint(asuint(flags_at) + asuint(asint(asuint(asint(asuint(asint(asuint(_e87) * asuint(n_2))) + asuint(rr.y))) * asuint(n_2))))) + asuint(rr.x)))*4));
                local_9 = (_e97 != 0u);
            } else {
                local_9 = true;
            }
            bool _e101 = local_9;
            if (_e101) {
                uint _e102 = usable;
                int _e104 = k_1;
                usable = (_e102 | (1u << uint(_e104)));
            }
        }
    }
    const uint _e115 = entry_at(REGION, r.x, r.y);
    uint _e118 = usable;
    place = ConstructPlace(r, true, (_e115 != NOT_SUMMED), _e118);
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
    uint _e42 = layer_now;
    float _e46 = cameras[_e42].pyramid.w;
    uint _e50 = asuint(entries.Load((uint(asint(asuint(asint(asuint(row_1.entries_at) + asuint(asint(asuint(y_1) * asuint(row_1.across))))) + asuint(x_1))) + naga_f2u32(_e46))*4));
    return _e50;
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
    uint _e188 = layer_now;
    float _e192 = cameras[_e188].atlas.w;
    float3 origin = (float3(float((slot & 255u)), float(((slot >> 8u) & 255u)), float((slot >> 16u))) * _e192);
    float3 local_50 = float3(float((uint(x_3) & ((1u << wide_1) - 1u))), float((uint(y_3) & ((1u << wide_1) - 1u))), float((uint(base.z) & ((1u << tall) - 1u))));
    uint _e224 = layer_now;
    float4 _e227 = cameras[_e224].atlas;
    const Spot spot_4 = ConstructSpot(int(2), level_6, ((((origin + local_50) + f) + (0.5).xxx) / _e227.xyz));
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
            uint _e17 = layer_now;
            float _e21 = cameras[_e17].window.x;
            local_23 = (asint(asuint(_e12) - asuint(FIRST)) < naga_f2i32(_e21));
        } else {
            local_23 = false;
        }
        bool _e25 = local_23;
        if (_e25) {
            int _e30 = l_1;
            local_24 = ((place_2.usable & (1u << uint(asint(asuint(_e30) - asuint(FIRST))))) == 0u);
        } else {
            local_24 = false;
        }
        bool _e39 = local_24;
        if (_e39) {
        } else {
            break;
        }
        {
            int _e40 = l_1;
            l_1 = asint(asuint(_e40) + asuint(int(1)));
        }
    }
    uint _e45 = layer_now;
    float _e49 = cameras[_e45].layer.w;
    if ((_e49 < 0.5)) {
        int _e52 = l_1;
        const Spot spot_5 = ConstructSpot(int(1), _e52, (0.0).xxx);
        return spot_5;
    }
    int _e57 = l_1;
    keep_level(k_4, _e57);
    int _e58 = l_1;
    const Spot _e59 = spot_soft(k_4, place_2, _e58, p_4);
    const Spot spot_6 = _e59;
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

Fetched fetch_together(float3 at_4)
{
    float4 _e4 = t_solid.SampleLevel(soft, at_4, 0.0);
    float4 _e9 = t_colour.SampleLevel(soft, at_4, 0.0);
    float4 _e13 = t_plus.SampleLevel(soft, at_4, 0.0);
    float4 _e18 = t_minus.SampleLevel(soft, at_4, 0.0);
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

    uint _e4 = layer_now;
    float4 _e7 = cameras[_e4].sun;
    float3 sun_1 = _e7.xyz;
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
    float _e44 = light;
    return _e44;
}

float3 lit_(float3 colour, float light_1)
{
    uint _e4 = layer_now;
    float _e8 = cameras[_e4].sun.w;
    uint _e11 = layer_now;
    float _e15 = cameras[_e11].daylight.y;
    uint _e19 = layer_now;
    float _e23 = cameras[_e19].sun.w;
    uint _e28 = layer_now;
    float _e32 = cameras[_e28].daylight.x;
    return (colour * ((_e8 * _e15) + (((1.0 - _e23) * _e32) * light_1)));
}

float3 sunlit(float3 colour_1, float light_2)
{
    uint _e4 = layer_now;
    float _e8 = cameras[_e4].sun.w;
    uint _e14 = layer_now;
    float _e18 = cameras[_e14].daylight.x;
    return (((colour_1 * (1.0 - _e8)) * _e18) * light_2);
}

float3 sky_tone(float3 day, float3 d_4)
{
    float3 colour_2 = (float3)0;

    uint _e6 = layer_now;
    float _e10 = cameras[_e6].daylight.w;
    float3 night = ((day * NIGHT_SKY) * _e10);
    uint _e14 = layer_now;
    float _e18 = cameras[_e14].daylight.z;
    colour_2 = lerp(night, day, _e18);
    uint _e23 = layer_now;
    float _e27 = cameras[_e23].sun.z;
    float height = (asin(clamp(_e27, -1.0, 1.0)) * 57.29578);
    float dusk = exp(-(pow(((height + 2.0) / 4.0), 2.0)));
    uint _e44 = layer_now;
    float4 _e47 = cameras[_e44].sun;
    float3 flat_sun = normalize((float3(_e47.xy, 0.0) + float3(1e-6, 0.0, 0.0)));
    float3 flat_d = normalize((float3(d_4.xy, 0.0) + float3(1e-6, 0.0, 0.0)));
    float towards = pow(max(dot(flat_d, flat_sun), 0.0), 3.0);
    float low_1 = pow((1.0 - clamp(abs(d_4.z), 0.0, 1.0)), 6.0);
    float3 _e80 = colour_2;
    uint _e84 = layer_now;
    float _e88 = cameras[_e84].daylight.z;
    colour_2 = lerp(_e80, (DUSK * max(_e88, 0.15)), (((dusk * towards) * low_1) * 0.6));
    float3 _e97 = colour_2;
    return _e97;
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

    uint _e15 = layer_now;
    float _e19 = cameras[_e15].reach.x;
    first = _e19;
    uint _e23 = layer_now;
    float _e27 = cameras[_e23].reach.x;
    touched = _e27;
    t_1 = (max(begin, 0.5) + phase);
    float _e37 = t_1;
    last_air = _e37;
    const float _e50 = world_top();
    float ceiling = (_e50 + 32.0);
    float surface_step = (0.25 / clamp(abs(d_6.z), 0.25, 1.0));
    uint _e62 = layer_now;
    float4 _e65 = cameras[_e62].water;
    float3 water_colour = _e65.xyz;
    const Kept _e88 = kept_none();
    kept = _e88;
    const Kept _e90 = kept_none();
    kept_next = _e90;
    const Row _e92 = row_none();
    row_own = _e92;
    const Row _e94 = row_none();
    row_up = _e94;
    const Row _e96 = row_none();
    row_far = _e96;
    uint _e108 = layer_now;
    float _e112 = cameras[_e108].reach.x;
    float last = min(_e112, end);
    uint _e116 = layer_now;
    float _e120 = cameras[_e116].pyramid.z;
    int most = naga_f2i32(_e120);
    uint2 loop_bound_6 = uint2(4294967295u, 4294967295u);
    bool loop_init_4 = true;
    while(true) {
        if (all(loop_bound_6 == uint2(0u, 0u))) { break; }
        loop_bound_6 -= uint2(loop_bound_6.y == 0u, 1u);
        if (!loop_init_4) {
            int _e761 = n_1;
            n_1 = asint(asuint(_e761) + asuint(int(1)));
        }
        loop_init_4 = false;
        int _e124 = n_1;
        if ((_e124 < most)) {
        } else {
            break;
        }
        {
            float _e126 = t_1;
            if (!((_e126 > last))) {
                float _e131 = through;
                local_28 = (_e131 < 0.01);
            } else {
                local_28 = true;
            }
            bool _e135 = local_28;
            if (_e135) {
                break;
            }
            int _e136 = n_1;
            steps = asint(asuint(_e136) + asuint(int(1)));
            float _e139 = t_1;
            float3 p_10 = (o_3 + (d_6 * _e139));
            if ((p_10.z > ceiling)) {
                local_29 = (d_6.z >= 0.0);
            } else {
                local_29 = false;
            }
            bool _e150 = local_29;
            if (_e150) {
                break;
            }
            if ((p_10.z < -8.0)) {
                local_30 = (d_6.z <= 0.0);
            } else {
                local_30 = false;
            }
            bool _e160 = local_30;
            if (_e160) {
                floored = true;
                break;
            }
            float _e162 = t_1;
            float _e163 = lv_from;
            if (!((_e162 < _e163))) {
                float _e168 = t_1;
                float _e169 = lv_to;
                local_31 = (_e168 >= _e169);
            } else {
                local_31 = true;
            }
            bool _e172 = local_31;
            if (_e172) {
                float _e173 = t_1;
                const int _e174 = level_at(_e173);
                lv = _e174;
                float _e175 = t_1;
                uint _e178 = layer_now;
                float _e182 = cameras[_e178].view.z;
                float k_10 = floor(log2(max((_e175 * _e182), 1.0)));
                uint _e191 = layer_now;
                float _e195 = cameras[_e191].view.z;
                lv_from = ((k_10 < 0.5) ? 0.0 : ((exp2(k_10) / _e195) * 1.00001));
                uint _e208 = layer_now;
                float _e212 = cameras[_e208].view.z;
                lv_to = ((exp2((k_10 + 1.0)) / _e212) * 0.99999);
                int _e216 = lv;
                uint _e220 = layer_now;
                float _e224 = cameras[_e220].reach.y;
                uint _e228 = layer_now;
                float _e232 = cameras[_e228].fade.z;
                uint _e237 = layer_now;
                float _e241 = cameras[_e237].view.z;
                lv_hand = ((exp2(((float(_e216) - _e224) + _e232)) / _e241) * 0.99999);
            }
            int _e245 = lv;
            level_12 = _e245;
            int _e247 = level_12;
            const int _e248 = ring_level(_e247, p_10);
            level_12 = _e248;
            keep_place(place_4, p_10);
            int _e249 = level_12;
            if ((_e249 > REGION)) {
                int _e253 = level_12;
                float2 c_2 = floor((p_10.xy / (float((int(1) << uint(_e253)))).xx));
                int _e261 = level_12;
                const uint _e266 = entry_at(_e261, naga_f2i32(c_2.x), naga_f2i32(c_2.y));
                if ((_e266 == NOT_SUMMED)) {
                    level_12 = REGION;
                }
            }
            int _e270 = level_12;
            if ((_e270 <= REGION)) {
                bool _e274 = place_4.summed;
                if (!(_e274)) {
                    int2 _e279 = place_4.region;
                    const float _e282 = leave2_(o_3.xy, d_6.xy, float2(_e279), 1024.0);
                    float _e283 = t_1;
                    t_1 = (max(_e282, _e283) + 0.01);
                    float _e287 = t_1;
                    last_air = _e287;
                    walking = false;
                    chained = false;
                    continue;
                }
            }
            skipped = false;
            int _e292 = level_12;
            float fine = float((int(1) << uint(_e292)));
            k_7 = int(0);
            uint2 loop_bound_7 = uint2(4294967295u, 4294967295u);
            bool loop_init_5 = true;
            while(true) {
                if (all(loop_bound_7 == uint2(0u, 0u))) { break; }
                loop_bound_7 -= uint2(loop_bound_7.y == 0u, 1u);
                if (!loop_init_5) {
                    int _e385 = k_7;
                    k_7 = asint(asuint(_e385) + asuint(int(1)));
                }
                loop_init_5 = false;
                int _e299 = k_7;
                if ((_e299 < int(3))) {
                    bool _e304 = in_air;
                    local_32 = _e304;
                } else {
                    local_32 = false;
                }
                bool _e306 = local_32;
                if (_e306) {
                    bool _e309 = walking;
                    local_33 = !(_e309);
                } else {
                    local_33 = false;
                }
                bool _e312 = local_33;
                if (_e312) {
                } else {
                    break;
                }
                {
                    int _e313 = level_12;
                    int _e316 = k_7;
                    int up_2 = min(asint(asuint(asint(asuint(_e313) + asuint(int(4)))) - asuint(asint(asuint(int(2)) * asuint(_e316)))), TOP);
                    float up_size = float((int(1) << uint(up_2)));
                    float2 cc = floor((p_10.xy / (up_size).xx));
                    row_3 = ZeroValueRow();
                    int _e332 = k_7;
                    if ((_e332 == int(0))) {
                        keep_row(row_far, up_2);
                        Row _e335 = row_far;
                        row_3 = _e335;
                    } else {
                        int _e336 = k_7;
                        if ((_e336 == int(1))) {
                            keep_row(row_up, up_2);
                            Row _e339 = row_up;
                            row_3 = _e339;
                        } else {
                            keep_row(row_own, up_2);
                            Row _e340 = row_own;
                            row_3 = _e340;
                        }
                    }
                    Row _e341 = row_3;
                    const uint _e346 = top_in(_e341, naga_f2i32(cc.x), naga_f2i32(cc.y));
                    float top = ((floor(((float(_e346) - 1.0) / fine)) + 2.0) * fine);
                    if ((p_10.z <= top)) {
                        continue;
                    }
                    const float _e359 = leave2_(o_3.xy, d_6.xy, cc, up_size);
                    next = _e359;
                    if ((d_6.z < 0.0)) {
                        float _e364 = next;
                        float _e365 = t_1;
                        next = min(_e364, (_e365 + ((top - p_10.z) / d_6.z)));
                    }
                    float _e372 = next;
                    float _e373 = t_1;
                    if ((_e372 > (_e373 + 0.01))) {
                        float _e377 = next;
                        t_1 = (_e377 + 0.01);
                        float _e380 = t_1;
                        last_air = _e380;
                        walking = false;
                        chained = false;
                        skipped = true;
                        break;
                    }
                }
            }
            bool _e387 = skipped;
            if (_e387) {
                continue;
            }
            over = 0.0;
            int _e390 = level_12;
            if ((_e390 < TOP)) {
                float _e395 = t_1;
                float _e396 = lv_hand;
                if (!((_e395 > _e396))) {
                    int _e401 = level_12;
                    int _e402 = lv;
                    local_35 = (_e401 != _e402);
                } else {
                    local_35 = true;
                }
                bool _e405 = local_35;
                local_34 = _e405;
            } else {
                local_34 = false;
            }
            bool _e407 = local_34;
            if (_e407) {
                float _e408 = t_1;
                uint _e411 = layer_now;
                float _e415 = cameras[_e411].view.z;
                uint _e422 = layer_now;
                float _e426 = cameras[_e422].reach.y;
                int _e428 = level_12;
                over = ((log2(max((_e408 * _e415), 1.0)) + _e426) - float(_e428));
            }
            float _e431 = over;
            uint _e434 = layer_now;
            float _e438 = cameras[_e434].fade.z;
            uint _e442 = layer_now;
            float _e446 = cameras[_e442].fade.z;
            mixed = ((_e431 - _e438) / (1.0 - _e446));
            int _e451 = level_12;
            if ((_e451 < FIRST)) {
                float _e454 = mixed;
                int _e455 = level_12;
                const float _e456 = in_ring(_e455, p_10);
                int _e458 = level_12;
                mixed = max(_e454, (1.0 - (_e456 / (RING_FADE * float((int(1) << uint(_e458)))))));
            }
            look_5 = ZeroValueLook();
            float _e470 = mixed;
            if ((_e470 > 0.0)) {
                Place _e473 = place_4;
                int _e474 = level_12;
                const Spot _e475 = spot_kept(kept, _e473, _e474, p_10);
                Place _e476 = place_4;
                int _e477 = level_12;
                const Spot _e480 = spot_kept(kept_next, _e476, asint(asuint(_e477) + asuint(int(1))), p_10);
                if ((_e475.kind == int(2))) {
                    local_36 = (_e480.kind == int(2));
                } else {
                    local_36 = false;
                }
                bool _e490 = local_36;
                if (_e490) {
                    const Fetched _e492 = fetch_together(_e475.at);
                    const Fetched _e494 = fetch_together(_e480.at);
                    const Look _e495 = look_of(_e492);
                    look_5 = _e495;
                    Look _e496 = look_5;
                    const Look _e497 = look_of(_e494);
                    float _e498 = mixed;
                    const Look _e499 = mix_looks(_e496, _e497, _e498);
                    look_5 = _e499;
                } else {
                    const Look _e500 = look_at(_e475, p_10);
                    look_5 = _e500;
                    bool _e502 = look_5.known;
                    if (_e502) {
                        const Look _e503 = look_at(_e480, p_10);
                        if (_e503.known) {
                            Look _e505 = look_5;
                            float _e506 = mixed;
                            const Look _e507 = mix_looks(_e505, _e503, _e506);
                            look_5 = _e507;
                        }
                    }
                }
            } else {
                Place _e508 = place_4;
                int _e509 = level_12;
                const Look _e510 = far_blend_kept(kept, _e508, _e509, p_10);
                look_5 = _e510;
            }
            int _e511 = level_12;
            float size_3 = float((int(1) << uint(_e511)));
            float _e517 = look_5.solid;
            float _e519 = look_5.water;
            bool _e522 = look_5.known;
            float thick = (_e522 ? max(_e517, _e519) : 0.0);
            if ((thick > 0.0)) {
                bool _e529 = walking;
                local_37 = !(_e529);
            } else {
                local_37 = false;
            }
            bool _e532 = local_37;
            if (_e532) {
                float _e535 = t_1;
                float _e536 = last_air;
                local_38 = ((_e535 - _e536) > (size_3 * 0.26));
            } else {
                local_38 = false;
            }
            bool _e542 = local_38;
            if (_e542) {
                float _e543 = t_1;
                walk_to = _e543;
                float _e544 = last_air;
                t_1 = (_e544 + ((phase * size_3) * 0.25));
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
                uint _e562 = layer_now;
                float _e566 = cameras[_e562].position.w;
                dt = ((thick > 0.02) ? (size_3 * surface_step) : (size_3 * _e566));
            } else {
                bool _e572 = walking;
                if (_e572) {
                    float _e575 = t_1;
                    float _e576 = walk_to;
                    local_39 = (_e575 < _e576);
                } else {
                    local_39 = false;
                }
                bool _e579 = local_39;
                if (_e579) {
                    in_air = false;
                    dt = (size_3 * surface_step);
                } else {
                    float _e582 = t_1;
                    last_air = _e582;
                    walking = false;
                    if ((d_6.z < 0.0)) {
                        float _e587 = dt;
                        dt = min(_e587, max((p_10.z / -(d_6.z)), (size_3 * 0.25)));
                    }
                }
            }
            bool _e597 = look_5.known;
            if (_e597) {
                bool _e600 = chained;
                local_40 = _e600;
            } else {
                local_40 = false;
            }
            bool _e602 = local_40;
            if (_e602) {
                bool _e606 = before.known;
                local_41 = _e606;
            } else {
                local_41 = false;
            }
            bool _e608 = local_41;
            if (_e608) {
                float _e609 = t_1;
                float _e610 = before_t;
                float length_ = (_e609 - _e610);
                float _e613 = before.solid;
                float _e615 = look_5.solid;
                float solid_share = (0.5 * (_e613 + _e615));
                float _e620 = before.water;
                float _e622 = look_5.water;
                float water_share = (0.5 * (_e620 + _e622));
                float solid = (1.0 - pow((1.0 - min(solid_share, 0.999)), (length_ / size_3)));
                uint _e636 = layer_now;
                float _e640 = cameras[_e636].water.w;
                float water = (1.0 - exp(((-(_e640) * water_share) * length_)));
                float covered = (1.0 - ((1.0 - solid) * (1.0 - water)));
                if ((covered > 0.0)) {
                    float _e656 = touched;
                    float _e657 = before_t;
                    touched = min(_e656, _e657);
                    Look _e659 = before;
                    const float _e660 = faces_light(_e659, d_6);
                    Look _e661 = look_5;
                    const float _e662 = faces_light(_e661, d_6);
                    float _e664 = before.solid;
                    float _e666 = look_5.solid;
                    float solids = max((_e664 + _e666), 1e-6);
                    float3 _e671 = before.colour;
                    const float3 _e672 = lit_(_e671, _e660);
                    float _e674 = before.solid;
                    float3 _e677 = look_5.colour;
                    const float3 _e678 = lit_(_e677, _e662);
                    float _e680 = look_5.solid;
                    float3 ground = (((_e672 * _e674) + (_e678 * _e680)) / (solids).xxx);
                    float3 _e686 = before.colour;
                    const float3 _e687 = sunlit(_e686, _e660);
                    float _e689 = before.solid;
                    float3 _e692 = look_5.colour;
                    const float3 _e693 = sunlit(_e692, _e662);
                    float _e695 = look_5.solid;
                    float3 ground_sun = (((_e687 * _e689) + (_e693 * _e695)) / (solids).xxx);
                    float both = max((solid + water), 1e-6);
                    const float3 _e705 = lit_(water_colour, 1.0);
                    float3 here_1 = (((ground * solid) + (_e705 * water)) / (both).xxx);
                    const float3 _e712 = sunlit(water_colour, 1.0);
                    float3 here_sun = (((ground_sun * solid) + (_e712 * water)) / (both).xxx);
                    float3 _e717 = colour_3;
                    float _e718 = through;
                    colour_3 = (_e717 + ((_e718 * covered) * here_1));
                    float3 _e722 = sunny;
                    float _e723 = through;
                    sunny = (_e722 + ((_e723 * covered) * here_sun));
                    if ((p_10.z <= (size_3 * 1.5))) {
                        bottom = true;
                        bottom_here = here_1;
                        bottom_sun = here_sun;
                        float _e732 = t_1;
                        bottom_t = _e732;
                        int _e733 = level_12;
                        bottom_level = _e733;
                    }
                    float _e734 = through;
                    if ((_e734 > 0.5)) {
                        float _e739 = through;
                        local_42 = ((_e739 * (1.0 - covered)) <= 0.5);
                    } else {
                        local_42 = false;
                    }
                    bool _e746 = local_42;
                    if (_e746) {
                        float _e747 = t_1;
                        first = _e747;
                        int _e748 = level_12;
                        first_level = _e748;
                    }
                    float _e749 = through;
                    through = (_e749 * (1.0 - covered));
                }
            }
            Look _e753 = look_5;
            before = _e753;
            float _e754 = t_1;
            before_t = _e754;
            bool _e756 = look_5.known;
            chained = _e756;
            float _e757 = t_1;
            float _e758 = dt;
            t_1 = (_e757 + _e758);
        }
    }
    bool _e763 = floored;
    if (_e763) {
        bool _e766 = bottom;
        local_43 = _e766;
    } else {
        local_43 = false;
    }
    bool _e768 = local_43;
    if (_e768) {
        float3 _e769 = colour_3;
        float _e770 = through;
        float3 _e771 = bottom_here;
        colour_3 = (_e769 + (_e770 * _e771));
        float3 _e774 = sunny;
        float _e775 = through;
        float3 _e776 = bottom_sun;
        sunny = (_e774 + (_e775 * _e776));
        through = 0.0;
        float _e780 = first;
        uint _e783 = layer_now;
        float _e787 = cameras[_e783].reach.x;
        if ((_e780 >= _e787)) {
            float _e789 = bottom_t;
            first = _e789;
            int _e790 = bottom_level;
            first_level = _e790;
        }
    }
    float3 gathered = colour_3;
    float3 _e792 = colour_3;
    float _e793 = through;
    const float3 _e794 = sky(d_6);
    colour_3 = (_e792 + (_e793 * _e794));
    float _e799 = first;
    uint _e802 = layer_now;
    float _e806 = cameras[_e802].reach.x;
    if ((_e799 < _e806)) {
        int _e808 = first_level;
        int _e815 = first_level;
        float read = ((_e815 >= int(0)) ? float((int(1) << uint(max(_e808, int(0))))) : 1.0);
        float _e820 = first;
        uint _e823 = layer_now;
        float _e827 = cameras[_e823].view.z;
        wide = max((_e820 * _e827), read);
    }
    float3 _e830 = colour_3;
    float _e831 = first;
    int _e832 = steps;
    int _e834 = first_level;
    float3 _e835 = sunny;
    float _e836 = first;
    float _e839 = wide;
    float3 _e840 = sunny;
    float _e842 = touched;
    float _e843 = through;
    const Seen seen_2 = ConstructSeen(_e830, _e831, float(_e832), _e834, _e835, (o_3 + (d_6 * _e836)), _e839, (gathered - _e840), _e842, _e843);
    return seen_2;
}

float scatter(uint2 at_5)
{
    uint h = (uint)0;

    h = ((at_5.x * 2654435761u) ^ (at_5.y * 2246822519u));
    uint _e9 = h;
    uint _e10 = h;
    h = ((_e9 ^ (_e10 >> 15u)) * 739982445u);
    uint _e16 = h;
    uint _e17 = h;
    h = (_e16 ^ (_e17 >> 12u));
    uint _e21 = h;
    return (float((_e21 & 65535u)) / 65536.0);
}

float weave(uint2 at_6, uint bits)
{
    uint v_3 = 0u;
    uint b_1 = 0u;

    uint x_5 = (at_6.x ^ at_6.y);
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
            v_3 = (((_e11 << 2u) | (((x_5 >> _e14) & 1u) << 1u)) | ((at_6.y >> _e22) & 1u));
        }
    }
    uint _e30 = v_3;
    return ((float(_e30) + 0.5) / float((1u << (2u * bits))));
}

uint2 naga_mod(uint2 lhs, uint2 rhs) {
    return lhs % (rhs == 0u ? 1u : rhs);
}

uint2 NagaDimensions2D(Texture2D<float4> tex)
{
    uint4 ret;
    tex.GetDimensions(0, ret.x, ret.y, ret.z);
    return ret.xy;
}

float blue(uint2 at_7)
{
    uint2 side = NagaDimensions2D(t_blue);
    float4 _e7 = t_blue.Load(int3(int2(naga_mod(at_7, side)), int(0)));
    return (((_e7.x * 255.0) + 0.5) / 256.0);
}

float ray_start(uint2 at_8)
{
    float phase_1 = (float)0;
    bool local_44 = (bool)0;

    uint _e3 = layer_now;
    float _e7 = cameras[_e3].ray_start.x;
    uint pattern = naga_f2u32(_e7);
    const float _e11 = scatter((at_8 * 2u));
    phase_1 = _e11;
    if (!((pattern == 1u))) {
        local_44 = (pattern == 3u);
    } else {
        local_44 = true;
    }
    bool _e21 = local_44;
    if (_e21) {
        uint _e24 = layer_now;
        float _e28 = cameras[_e24].ray_start.y;
        const float _e30 = weave(at_8, naga_f2u32(_e28));
        phase_1 = _e30;
        if ((pattern == 3u)) {
            float _e33 = phase_1;
            uint _e36 = layer_now;
            float _e40 = cameras[_e36].ray_start.z;
            const float _e41 = blue(at_8);
            phase_1 = frac((_e33 + (_e40 * _e41)));
        }
    } else {
        if ((pattern == 2u)) {
            const float _e47 = blue(at_8);
            phase_1 = _e47;
        }
    }
    float _e48 = phase_1;
    uint _e51 = layer_now;
    float _e55 = cameras[_e51].ray_start.w;
    return (_e48 * _e55);
}

float worth_of(float near, int level_13)
{
    bool local_45 = (bool)0;

    uint _e4 = layer_now;
    float _e8 = cameras[_e4].layer.y;
    if ((_e8 < 0.5)) {
        return 0.0;
    }
    uint _e14 = layer_now;
    float _e18 = cameras[_e14].layer.z;
    if ((_e18 < 0.5)) {
        return 1.0;
    }
    uint _e24 = layer_now;
    float _e28 = cameras[_e24].reach.x;
    if (!((near >= _e28))) {
        local_45 = (level_13 < int(0));
    } else {
        local_45 = true;
    }
    bool _e36 = local_45;
    if (_e36) {
        return 0.0;
    }
    float cell = float((int(1) << uint(level_13)));
    uint _e44 = layer_now;
    float _e48 = cameras[_e44].view.z;
    uint _e52 = layer_now;
    float _e56 = cameras[_e52].pyramid.y;
    float ratio = (((near * _e48) * _e56) / cell);
    uint _e61 = layer_now;
    float _e65 = cameras[_e61].fade.x;
    uint _e71 = layer_now;
    float _e75 = cameras[_e71].fade.y;
    return clamp((((ratio / _e65) - 1.0) / max((exp2(_e75) - 1.0), 0.001)), 0.0, 1.0);
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
    float2 at_9 = float2(float((_e17 & int(65535))), float((_e17 >> 16u)));
    float2 c_3 = clamp(((p_8.xy / (size_4).xx) - least), (0.5).xx, ((across - 0.5)).xx);
    float2 uv = ((c_3 + at_9) / float2(NagaDimensions2D(t_shadow)));
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
    uint _e13 = layer_now;
    float _e17 = cameras[_e13].view.w;
    if ((_e17 > 0.5)) {
        const float2 _e22 = cell_light(seen.at, seen.wide);
        uint _e26 = layer_now;
        float _e30 = cameras[_e26].follow.y;
        sun = ((_e30 > 0.5) ? _e22.x : 1.0);
        sky_open = _e22.y;
    }
    float3 _e36 = colour_4;
    float _e38 = sun;
    colour_4 = (_e36 - (seen.sunny * (1.0 - _e38)));
    float _e43 = sky_open;
    uint _e46 = layer_now;
    float _e50 = cameras[_e46].sky.y;
    uint _e55 = layer_now;
    float _e59 = cameras[_e55].sky.x;
    float sky_2 = ((_e59 > 0.5) ? lerp(1.0, _e43, _e50) : 1.0);
    uint _e69 = layer_now;
    float _e73 = cameras[_e69].sky.z;
    float3 tint = lerp((1.0).xxx, SKY_TINT, _e73);
    float3 _e75 = colour_4;
    return (_e75 + (seen.skyish * ((sky_2 * tint) - (1.0).xxx)));
}

float3 fogged(float3 colour_5, float3 d_7, float t_2)
{
    float along = (float)0;

    uint _e5 = layer_now;
    float _e9 = cameras[_e5].fog.x;
    if ((_e9 <= 0.0)) {
        return colour_5;
    }
    uint _e14 = layer_now;
    float thins = cameras[_e14].fog.y;
    uint _e21 = layer_now;
    float _e25 = cameras[_e21].fog.x;
    uint _e29 = layer_now;
    float _e33 = cameras[_e29].position.z;
    uint _e36 = layer_now;
    float _e40 = cameras[_e36].fog.z;
    float at_eye = (_e25 * exp(clamp((-(thins) * (_e33 - _e40)), -80.0, 80.0)));
    float climb = (thins * d_7.z);
    along = t_2;
    if ((abs((climb * t_2)) > 0.0001)) {
        along = ((1.0 - exp(-(clamp((climb * t_2), -80.0, 80.0)))) / climb);
    }
    float _e65 = along;
    float amount = (1.0 - exp((-(at_eye) * _e65)));
    uint _e72 = layer_now;
    float4 _e75 = cameras[_e72].sun;
    uint _e84 = layer_now;
    float _e88 = cameras[_e84].fog.w;
    uint _e92 = layer_now;
    float _e96 = cameras[_e92].daylight.x;
    float sunward = ((pow(max(dot(d_7, _e75.xyz), 0.0), 8.0) * _e88) * _e96);
    const float3 _e101 = sky_tone(lerp(HORIZON, FOG_WARM, sunward), d_7);
    return lerp(colour_5, _e101, amount);
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

    uint _e3 = layer_now;
    float4 _e6 = cameras[_e3].view;
    uint2 size_5 = naga_f2u32(_e6.xy);
    if (!((id_1.x >= size_5.x))) {
        local_46 = (id_1.y >= size_5.y);
    } else {
        local_46 = true;
    }
    bool _e19 = local_46;
    if (_e19) {
        return;
    }
    uint _e29 = layer_now;
    float _e33 = cameras[_e29].pyramid.x;
    uint _e38 = layer_now;
    float _e42 = cameras[_e38].layer.x;
    uint2 parent = min(naga_f2u32(((float2(id_1.xy) + (0.5).xx) * _e33)), uint2((naga_f2u32(max(_e42, 1.0)) - 1u), 65535u));
    uint _e53 = layer_now;
    float _e57 = cameras[_e53].layer.x;
    if ((_e57 > 0.0)) {
        uint _e63 = layer_now;
        float _e67 = cameras[_e63].layer.x;
        const uint _e72 = get_worth(((parent.y * naga_f2u32(_e67)) + parent.x));
        uint worth = (_e72 & 255u);
        shown = (float(worth) / 255.0);
        if ((worth == 0u)) {
            put_picture(int2(id_1.xy), (0.0).xxxx);
            uint _e88 = layer_now;
            float _e92 = cameras[_e88].reach.x;
            put_far(int2(id_1.xy), float4(_e92, 0.0, 0.0, 0.0));
            uint _e99 = layer_now;
            float _e103 = cameras[_e99].layer.y;
            if ((_e103 > 0.5)) {
                put_worth(((id_1.y * size_5.x) + id_1.x), 0u);
            }
            return;
        }
    }
    uint _e116 = layer_now;
    float _e120 = cameras[_e116].tiles.z;
    if ((_e120 > 0.0)) {
        uint _e127 = layer_now;
        float _e131 = cameras[_e127].tiles.z;
        uint2 tile = naga_f2u32((float2(id_1.xy) / (_e131).xx));
        uint _e139 = layer_now;
        float _e143 = cameras[_e139].tiles.x;
        float _e149 = asfloat(starts.Load(((tile.y * naga_f2u32(_e143)) + tile.x)*4));
        begin_1 = _e149;
    }
    uint _e152 = layer_now;
    float _e156 = cameras[_e152].layer.x;
    if ((_e156 > 0.0)) {
        uint _e163 = layer_now;
        float _e167 = cameras[_e163].follow.x;
        local_47 = (_e167 > 0.5);
    } else {
        local_47 = false;
    }
    bool _e171 = local_47;
    if (_e171) {
        int2 above = int2(parent);
        const int2 _e173 = far_above_edge();
        uint _e176 = layer_now;
        float _e180 = cameras[_e176].reach.x;
        nearest = _e180;
        uint2 loop_bound_9 = uint2(4294967295u, 4294967295u);
        bool loop_init_7 = true;
        while(true) {
            if (all(loop_bound_9 == uint2(0u, 0u))) { break; }
            loop_bound_9 -= uint2(loop_bound_9.y == 0u, 1u);
            if (!loop_init_7) {
                int _e206 = k_8;
                k_8 = asint(asuint(_e206) + asuint(int(1)));
            }
            loop_init_7 = false;
            int _e184 = k_8;
            if ((_e184 < int(9))) {
            } else {
                break;
            }
            {
                int _e187 = k_8;
                int _e192 = k_8;
                int2 at_10 = clamp(asint(asuint(above) + asuint(int2(asint(asuint(naga_mod(_e187, int(3))) - asuint(int(1))), asint(asuint(naga_div(_e192, int(3))) - asuint(int(1)))))), (int(0)).xx, _e173);
                float _e202 = nearest;
                const float _e203 = get_far_above(at_10);
                nearest = min(_e202, _e203);
            }
        }
        float _e208 = begin_1;
        float _e209 = nearest;
        float _e212 = nearest;
        uint _e215 = layer_now;
        float _e219 = cameras[_e215].view.z;
        begin_1 = max(_e208, ((_e209 - 2.0) - ((_e212 * _e219) * 4.0)));
    }
    float _e225 = begin_1;
    uint _e228 = layer_now;
    float _e232 = cameras[_e228].sky.w;
    begin_1 = max(_e225, _e232);
    float2 uv_1 = ((float2(id_1.xy) + (0.5).xx) / float2(size_5));
    uint _e243 = layer_now;
    float4 _e246 = cameras[_e243].forward;
    uint _e255 = layer_now;
    float4 _e258 = cameras[_e255].right;
    uint _e269 = layer_now;
    float4 _e272 = cameras[_e269].up;
    float3 d_8 = normalize(((_e246.xyz + (((uv_1.x * 2.0) - 1.0) * _e258.xyz)) + ((1.0 - (uv_1.y * 2.0)) * _e272.xyz)));
    uint _e279 = layer_now;
    float _e283 = cameras[_e279].reach.x;
    end_1 = _e283;
    uint _e287 = layer_now;
    float _e291 = cameras[_e287].follow.w;
    if ((_e291 > 0.5)) {
        uint _e296 = layer_now;
        float4 _e299 = cameras[_e296].position;
        const float _e301 = map_end(_e299.xyz, d_8);
        end_1 = _e301;
    }
    uint _e304 = layer_now;
    float4 _e307 = cameras[_e304].position;
    const float _e310 = ray_start(id_1.xy);
    float _e311 = begin_1;
    float _e312 = end_1;
    const Seen _e313 = march(_e307.xyz, d_8, _e310, _e311, _e312);
    const float _e316 = worth_of(_e313.distance_, _e313.level);
    bool hidden = (round((_e316 * 255.0)) >= 255.0);
    here = _e313.colour;
    if (!(hidden)) {
        const float3 _e325 = shaded(_e313);
        here = _e325;
    }
    uint _e331 = layer_now;
    float _e335 = cameras[_e331].reach.x;
    float met = ((_e313.distance_ >= _e335) ? _e313.touched : _e313.distance_);
    uint _e340 = layer_now;
    float _e344 = cameras[_e340].reach.x;
    if ((met < _e344)) {
        const float3 _e347 = sky(d_8);
        float3 behind = (_e313.sky_share * _e347);
        float covered_1 = max((1.0 - _e313.sky_share), 0.0001);
        float3 _e354 = here;
        const float3 _e358 = fogged(((_e354 - behind) / (covered_1).xxx), d_8, met);
        uint _e361 = layer_now;
        float _e365 = cameras[_e361].fade.w;
        float shown_2 = pow(covered_1, (1.0 + ((_e365 * _e313.sky_share) * _e313.sky_share)));
        const float3 _e373 = sky(d_8);
        here = ((_e373 * (1.0 - shown_2)) + (_e358 * shown_2));
    }
    uint _e381 = layer_now;
    float _e385 = cameras[_e381].layer.y;
    if ((_e385 > 0.5)) {
        const float _e395 = worth_of(_e313.distance_, _e313.level);
        put_worth(((id_1.y * size_5.x) + id_1.x), naga_f2u32(round((_e395 * 255.0))));
    }
    float3 _e400 = here;
    colour_6 = _e400;
    uint _e404 = layer_now;
    float _e408 = cameras[_e404].reach.z;
    if ((_e408 > 1.5)) {
        local_48 = (_e313.level >= int(0));
    } else {
        local_48 = false;
    }
    bool _e417 = local_48;
    if (_e417) {
        float3 tints[6] = Constructarray6_float3_(float3(1.0, 0.3, 0.3), float3(0.3, 1.0, 0.3), float3(0.3, 0.4, 1.0), float3(1.0, 1.0, 0.3), float3(1.0, 0.3, 1.0), float3(0.3, 1.0, 1.0));
        float3 _e443 = colour_6;
        colour_6 = ((_e443 * 0.4) + (tints[min(uint(naga_mod(_e313.level, int(6))), 5u)] * 0.35));
    } else {
        uint _e455 = layer_now;
        float _e459 = cameras[_e455].reach.z;
        if ((_e459 > 0.5)) {
            uint _e466 = layer_now;
            float _e470 = cameras[_e466].reach.z;
            local_49 = (_e470 < 1.5);
        } else {
            local_49 = false;
        }
        bool _e474 = local_49;
        if (_e474) {
            float k_11 = clamp((_e313.steps / 256.0), 0.0, 1.0);
            colour_6 = ((k_11 < 0.5) ? lerp(float3(0.0, 0.0, 0.4), float3(0.0, 0.8, 0.2), (k_11 * 2.0)) : lerp(float3(0.0, 0.8, 0.2), float3(1.0, 0.0, 0.0), ((k_11 - 0.5) * 2.0)));
            float3 _e508 = colour_6;
            float3 _e509 = colour_6;
            colour_6 = (_e508 * _e509);
        }
    }
    float3 _e513 = colour_6;
    const float3 _e519 = to_srgb(clamp(_e513, (0.0).xxx, (1.0).xxx));
    float _e520 = shown;
    float _e522 = shown;
    put_picture(int2(id_1.xy), float4((_e519 * _e520), _e522));
    put_far(int2(id_1.xy), float4(_e313.touched, _e313.steps, 0.0, 0.0));
    return;
}

uint naga_div(uint lhs, uint rhs) {
    return lhs / (rhs == 0u ? 1u : rhs);
}

[numthreads(8, 8, 1)]
void march_all(uint3 id : SV_DispatchThreadID)
{
    uint2 at = (uint2)0;

    float _e5 = cameras[0].view.x;
    uint base_across = (naga_div((naga_f2u32(_e5) + 7u), 8u) * 8u);
    float _e17 = cameras[1].view.x;
    uint middle_across = (naga_div((naga_f2u32(_e17) + 7u), 8u) * 8u);
    at = id.xy;
    if ((id.x < base_across)) {
        layer_now = 0u;
    } else {
        if ((id.x < (base_across + middle_across))) {
            uint _e35 = at.x;
            at.x = (_e35 - base_across);
            layer_now = 1u;
        } else {
            uint _e40 = at.x;
            at.x = (_e40 - (base_across + middle_across));
            layer_now = 2u;
        }
    }
    uint2 _e45 = at;
    march_pixel(_e45);
    return;
}

struct NagaConstants {
    int first_vertex;
    int first_instance;
    uint other;
};
ConstantBuffer<NagaConstants> _NagaConstants: register(b2);

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
    float4 list;
    float4 cut;
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
    float reached;
    int reached_level;
    bool ended_in_air;
    float ended_at;
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

static const uint STACK = 13u;
static const uint NOT_CUT = 4294967295u;
static const int TOP = int(16);
static const float3 NIGHT_SKY = float3(0.35, 0.45, 0.8);
static const float3 DUSK = float3(1.0, 0.45, 0.18);
static const float3 HORIZON = float3(0.62, 0.72, 0.86);
static const float3 ZENITH = float3(0.16, 0.32, 0.66);
static const int FIRST = int(5);
static const int TABLE = int(12);
static const float3 SKY_TINT = float3(0.84, 1.01, 1.4);
static const float3 FOG_WARM = float3(1.0, 0.82, 0.62);
static const bool LOADED = true;
static const int REGION = int(10);
static const float WORLD_HEIGHT = 1024.0;
static const uint NO_SLOT = 4294967295u;
static const uint PAST_MAP = 4294836224u;
static const uint NOT_SUMMED = 4294901760u;
static const float RING_FADE = 8.0;
static const float3 ROCK = float3(0.25, 0.24, 0.22);
static const bool FACES = false;
static const uint NO_BRICK = 4294967295u;

cbuffer cameras : register(b0) { Camera cameras[3]; }
cbuffer levels : register(b1) { int4 levels[48]; }
ByteAddressBuffer entries : register(t0);
ByteAddressBuffer index : register(t1);
ByteAddressBuffer pool : register(t2);
RWTexture2D<unorm float4> picture0_ : register(u0);
RWTexture2D<unorm float4> picture1_ : register(u4);
RWTexture2D<unorm float4> picture2_ : register(u5);
RWTexture2D<float4> how_far : register(u1);
ByteAddressBuffer worth_in : register(t3);
RWByteAddressBuffer worth_out : register(u3);
Texture2D<float4> far_prev : register(t4);
RWByteAddressBuffer long_rays : register(u2);
RWByteAddressBuffer cut : register(u6);
static uint layer_now = (uint)0;
static uint listed_now = (uint)0;
Texture2D<float4> t_shadow : register(t10);
SamplerState nagaSamplerHeap[2048]: register(s0, space0);
SamplerComparisonState nagaComparisonSamplerHeap[2048]: register(s2048, space0);
StructuredBuffer<uint> nagaGroup0SamplerIndexArray : register(t13, space0);
static const SamplerState soft = nagaSamplerHeap[nagaGroup0SamplerIndexArray[0]];
Texture2D<float4> t_sky : register(t11);
Texture2D<float4> t_blue : register(t12);
ByteAddressBuffer slots : register(t5);
Texture3D<float4> t_solid : register(t6);
Texture3D<float4> t_colour : register(t7);
Texture3D<float4> t_plus : register(t8);
Texture3D<float4> t_minus : register(t9);

int naga_f2i32(float value) {
    return int(clamp(value, -2147483600.0, 2147483500.0));
}

float tile_start(uint2 tile, uint across)
{
    float _e8 = cameras[0].view.y;
    float4 _e14 = far_prev.Load(int3(asint(asuint(int2(tile)) + asuint(int2(int(0), naga_f2i32(_e8)))), int(0)));
    return _e14.x;
}

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

void put_worth(uint i_1, uint v_2)
{
    const uint _e4 = base_pixels();
    uint _e6 = layer_now;
    worth_out.Store((i_1 + ((_e6 == 1u) ? _e4 : 0u))*4, asuint(v_2));
    return;
}

uint get_worth(uint i_2)
{
    const uint _e3 = base_pixels();
    uint _e5 = layer_now;
    uint _e11 = asuint(worth_in.Load((i_2 + ((_e5 == 2u) ? _e3 : 0u))*4));
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

uint list_at(uint which, uint slot)
{
    float _e6 = cameras[0].list.z;
    return (((which * naga_f2u32(_e6)) + slot) * 3u);
}

uint befores_at(uint which_1, uint slot_1)
{
    float _e6 = cameras[0].list.z;
    uint holds = naga_f2u32(_e6);
    return ((6u * holds) + (((which_1 * holds) + slot_1) * 17u));
}

uint pieces_at(uint which_2)
{
    float _e6 = cameras[0].list.z;
    float _e13 = cameras[0].cut.y;
    return ((40u * naga_f2u32(_e6)) + (which_2 * naga_f2u32(_e13)));
}

uint own_stack(uint slot_2)
{
    float _e6 = cameras[0].list.z;
    float _e14 = cameras[0].cut.y;
    return (((40u * naga_f2u32(_e6)) + (2u * naga_f2u32(_e14))) + (slot_2 * STACK));
}

uint piece_stack(uint piece)
{
    float _e5 = cameras[0].list.z;
    const uint _e7 = own_stack(naga_f2u32(_e5));
    return (_e7 + (piece * STACK));
}

float list_long(uint2 id_1, float steps, bool joined)
{
    float long_ = cameras[0].list.y;
    if ((steps < (joined ? (long_ * 0.5) : long_))) {
        return 0.0;
    }
    float _e17 = cameras[0].list.x;
    uint now = naga_f2u32(_e17);
    uint _e22; long_rays.InterlockedAdd(now*4, 1u, _e22);
    float _e27 = cameras[0].list.z;
    if ((_e22 >= naga_f2u32(_e27))) {
        return 0.0;
    }
    const uint _e31 = list_at(now, _e22);
    uint _e35 = layer_now;
    cut.Store(_e31*4, asuint((((_e35 << 30u) | (id_1.y << 15u)) | id_1.x)));
    cut.Store((_e31 + 1u)*4, asuint(0u));
    cut.Store((_e31 + 2u)*4, asuint(0u));
    listed_now = _e22;
    return float((_e22 + 1u));
}

uint2 naga_f2u32(float2 value) {
    return uint2(clamp(value, 0.0, 4294967000.0));
}

uint3 listed_pixel(uint slot_3)
{
    bool local_1 = (bool)0;

    float _e5 = cameras[0].list.z;
    if ((slot_3 >= naga_f2u32(_e5))) {
        return uint3(0u, 0u, 3u);
    }
    float _e18 = cameras[0].list.x;
    const uint _e21 = list_at((1u - naga_f2u32(_e18)), slot_3);
    uint entry = asuint(cut.Load(_e21*4));
    uint layer_1 = (entry >> 30u);
    uint2 pixel = uint2((entry & 32767u), ((entry >> 15u) & 32767u));
    if (!((layer_1 == 3u))) {
        float4 _e41 = cameras[layer_1].view;
        local_1 = any((pixel >= naga_f2u32(_e41.xy)));
    } else {
        local_1 = true;
    }
    bool _e47 = local_1;
    if (_e47) {
        return uint3(0u, 0u, 3u);
    }
    const int _e54 = far_left(layer_1);
    float4 _e59 = far_prev.Load(int3(asint(asuint(int2(pixel)) + asuint(int2(_e54, int(0)))), int(0)));
    float listed = _e59.w;
    if ((naga_f2u32(listed) != (slot_3 + 1u))) {
        return uint3(0u, 0u, 3u);
    }
    return uint3(pixel, layer_1);
}

uint cut_slot(uint2 id_2)
{
    float _e5 = cameras[0].cut.x;
    if ((_e5 <= 0.0)) {
        return NOT_CUT;
    }
    uint _e12 = layer_now;
    const int _e13 = far_left(_e12);
    float4 _e18 = far_prev.Load(int3(asint(asuint(int2(id_2)) + asuint(int2(_e13, int(0)))), int(0)));
    float listed_1 = _e18.w;
    return ((listed_1 > 0.5) ? (naga_f2u32(listed_1) - 1u) : NOT_CUT);
}

uint cut_first(uint slot_4)
{
    float _e7 = cameras[0].list.x;
    const uint _e10 = list_at((1u - naga_f2u32(_e7)), slot_4);
    uint cuts = asuint(cut.Load((_e10 + 1u)*4));
    return ((cuts == 0u) ? 17u : (firstbitlow(cuts) + 1u));
}

float cut_before(uint slot_5, uint place)
{
    float _e8 = cameras[0].list.x;
    const uint _e11 = befores_at((1u - naga_f2u32(_e8)), slot_5);
    uint _e14 = asuint(cut.Load((_e11 + place)*4));
    return asfloat(_e14);
}

void put_began(uint at_4, float began, float starts)
{
    cut.Store((at_4 + 10u)*4, asuint(asuint(began)));
    cut.Store((at_4 + 11u)*4, asuint(asuint(starts)));
    return;
}

void put_stack(uint at_5, Seen seen)
{
    float3 gathered = (seen.skyish + seen.sunny);
    cut.Store(at_5*4, asuint(asuint(gathered.x)));
    cut.Store((at_5 + 1u)*4, asuint(asuint(gathered.y)));
    cut.Store((at_5 + 2u)*4, asuint(asuint(gathered.z)));
    cut.Store((at_5 + 3u)*4, asuint(asuint(seen.sunny.x)));
    cut.Store((at_5 + 4u)*4, asuint(asuint(seen.sunny.y)));
    cut.Store((at_5 + 5u)*4, asuint(asuint(seen.sunny.z)));
    cut.Store((at_5 + 6u)*4, asuint(asuint(seen.sky_share)));
    cut.Store((at_5 + 7u)*4, asuint(asuint(seen.reached)));
    cut.Store((at_5 + 8u)*4, asuint(asuint(seen.touched)));
    cut.Store((at_5 + 9u)*4, asuint((((naga_f2u32(seen.steps) << 5u) | uint(asint(asuint(seen.reached_level) + asuint(int(1))))) | (uint(seen.ended_in_air) << 31u))));
    cut.Store((at_5 + 12u)*4, asuint(asuint(seen.ended_at)));
    return;
}

uint2 parent_of(uint2 id_3)
{
    uint _e7 = layer_now;
    float _e11 = cameras[_e7].pyramid.x;
    uint _e16 = layer_now;
    float _e20 = cameras[_e16].layer.x;
    return min(naga_f2u32(((float2(id_3) + (0.5).xx) * _e11)), uint2((naga_f2u32(max(_e20, 1.0)) - 1u), 65535u));
}

float3 ray_through(uint2 id_4)
{
    uint _e7 = layer_now;
    float4 _e10 = cameras[_e7].view;
    float2 uv = ((float2(id_4) + (0.5).xx) / _e10.xy);
    uint _e15 = layer_now;
    float4 _e18 = cameras[_e15].forward;
    uint _e27 = layer_now;
    float4 _e30 = cameras[_e27].right;
    uint _e41 = layer_now;
    float4 _e44 = cameras[_e41].up;
    return normalize(((_e18.xyz + (((uv.x * 2.0) - 1.0) * _e30.xyz)) + ((1.0 - (uv.y * 2.0)) * _e44.xyz)));
}

float scatter(uint2 at_6)
{
    uint h = (uint)0;

    h = ((at_6.x * 2654435761u) ^ (at_6.y * 2246822519u));
    uint _e9 = h;
    uint _e10 = h;
    h = ((_e9 ^ (_e10 >> 15u)) * 739982445u);
    uint _e16 = h;
    uint _e17 = h;
    h = (_e16 ^ (_e17 >> 12u));
    uint _e21 = h;
    return (float((_e21 & 65535u)) / 65536.0);
}

float cut_place(uint place_1, float shift)
{
    if ((place_1 == 0u)) {
        return 0.0;
    }
    if ((place_1 > 16u)) {
        return 1e30;
    }
    uint _e10 = layer_now;
    float _e14 = cameras[_e10].cut.x;
    return (_e14 * exp2((((float(place_1) - 1.0) + shift) * 0.5)));
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

float3 sky_tone(float3 day, float3 d)
{
    float3 colour = (float3)0;

    uint _e6 = layer_now;
    float _e10 = cameras[_e6].daylight.w;
    float3 night = ((day * NIGHT_SKY) * _e10);
    uint _e14 = layer_now;
    float _e18 = cameras[_e14].daylight.z;
    colour = lerp(night, day, _e18);
    uint _e23 = layer_now;
    float _e27 = cameras[_e23].sun.z;
    float height = (asin(clamp(_e27, -1.0, 1.0)) * 57.29578);
    float dusk = exp(-(pow(((height + 2.0) / 4.0), 2.0)));
    uint _e44 = layer_now;
    float4 _e47 = cameras[_e44].sun;
    float3 flat_sun = normalize((float3(_e47.xy, 0.0) + float3(1e-6, 0.0, 0.0)));
    float3 flat_d = normalize((float3(d.xy, 0.0) + float3(1e-6, 0.0, 0.0)));
    float towards = pow(max(dot(flat_d, flat_sun), 0.0), 3.0);
    float low = pow((1.0 - clamp(abs(d.z), 0.0, 1.0)), 6.0);
    float3 _e80 = colour;
    uint _e84 = layer_now;
    float _e88 = cameras[_e84].daylight.z;
    colour = lerp(_e80, (DUSK * max(_e88, 0.15)), (((dusk * towards) * low) * 0.6));
    float3 _e97 = colour;
    return _e97;
}

float3 sky(float3 d_1)
{
    float up_1 = clamp(d_1.z, -0.2, 1.0);
    const float3 _e12 = sky_tone(lerp(HORIZON, ZENITH, pow(max(up_1, 0.0), 0.6)), d_1);
    return _e12;
}

float shadow_wide(float first, int first_level)
{
    uint _e4 = layer_now;
    float _e8 = cameras[_e4].reach.x;
    if ((first >= _e8)) {
        return 0.0;
    }
    float read = ((first_level >= int(0)) ? float((int(1) << uint(max(first_level, int(0))))) : 1.0);
    uint _e23 = layer_now;
    float _e27 = cameras[_e23].view.z;
    return max((first * _e27), read);
}

float worth_of(float near, int level)
{
    bool local_2 = (bool)0;

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
        local_2 = (level < int(0));
    } else {
        local_2 = true;
    }
    bool _e36 = local_2;
    if (_e36) {
        return 0.0;
    }
    float cell = float((int(1) << uint(level)));
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

int table(int level_1, int k)
{
    int row_4 = ((level_1 < FIRST) ? asint(asuint(int(12)) + asuint(asint(asuint(int(4)) - asuint(level_1)))) : asint(asuint(level_1) - asuint(FIRST)));
    uint at_11 = uint(asint(asuint(asint(asuint(row_4) * asuint(TABLE))) + asuint(k)));
    int _e22 = levels[(at_11 >> 2u)][(at_11 & 3u)];
    return _e22;
}

uint2 NagaDimensions2D(Texture2D<float4> tex)
{
    uint4 ret;
    tex.GetDimensions(0, ret.x, ret.y, ret.z);
    return ret.xy;
}

float2 light_of(int level_2, float3 p)
{
    float size_1 = float((int(1) << uint(level_2)));
    const int _e7 = table(level_2, int(1));
    const int _e10 = table(level_2, int(2));
    float2 least = float2(float(_e7), float(_e10));
    const int _e14 = table(level_2, int(3));
    float across_1 = float(_e14);
    const int _e17 = table(level_2, int(11));
    float2 at_12 = float2(float((_e17 & int(65535))), float((_e17 >> 16u)));
    float2 c_2 = clamp(((p.xy / (size_1).xx) - least), (0.5).xx, ((across_1 - 0.5)).xx);
    float2 uv_1 = ((c_2 + at_12) / float2(NagaDimensions2D(t_shadow)));
    float4 _e43 = t_shadow.SampleLevel(soft, uv_1, 0.0);
    float shadow = _e43.x;
    float4 _e48 = t_sky.SampleLevel(soft, uv_1, 0.0);
    float sky_1 = _e48.x;
    return float2(clamp((((p.z - shadow) / size_1) + 1.5), 0.0, 1.0), sky_1);
}

float2 cell_light(float3 p_1, float footprint)
{
    float f_1 = clamp(log2(max(footprint, 1.0)), 5.0, 16.0);
    int level_16 = naga_f2i32(floor(f_1));
    const float2 _e10 = light_of(level_16, p_1);
    if ((level_16 >= TOP)) {
        return _e10;
    }
    const float2 _e15 = light_of(asint(asuint(level_16) + asuint(int(1))), p_1);
    return lerp(_e10, _e15, (f_1 - floor(f_1)));
}

float3 shaded(Seen seen_1)
{
    float3 colour_1 = (float3)0;
    float sun = 1.0;
    float sky_open = 1.0;

    if ((seen_1.wide <= 0.0)) {
        return seen_1.colour;
    }
    colour_1 = seen_1.colour;
    uint _e13 = layer_now;
    float _e17 = cameras[_e13].view.w;
    if ((_e17 > 0.5)) {
        const float2 _e22 = cell_light(seen_1.at, seen_1.wide);
        uint _e26 = layer_now;
        float _e30 = cameras[_e26].follow.y;
        sun = ((_e30 > 0.5) ? _e22.x : 1.0);
        sky_open = _e22.y;
    }
    float3 _e36 = colour_1;
    float _e38 = sun;
    colour_1 = (_e36 - (seen_1.sunny * (1.0 - _e38)));
    float _e43 = sky_open;
    uint _e46 = layer_now;
    float _e50 = cameras[_e46].sky.y;
    uint _e55 = layer_now;
    float _e59 = cameras[_e55].sky.x;
    float sky_2 = ((_e59 > 0.5) ? lerp(1.0, _e43, _e50) : 1.0);
    uint _e69 = layer_now;
    float _e73 = cameras[_e69].sky.z;
    float3 tint = lerp((1.0).xxx, SKY_TINT, _e73);
    float3 _e75 = colour_1;
    return (_e75 + (seen_1.skyish * ((sky_2 * tint) - (1.0).xxx)));
}

float3 fogged(float3 colour_2, float3 d_2, float t_1)
{
    float along = (float)0;

    uint _e5 = layer_now;
    float _e9 = cameras[_e5].fog.x;
    if ((_e9 <= 0.0)) {
        return colour_2;
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
    float climb = (thins * d_2.z);
    along = t_1;
    if ((abs((climb * t_1)) > 0.0001)) {
        along = ((1.0 - exp(-(clamp((climb * t_1), -80.0, 80.0)))) / climb);
    }
    float _e65 = along;
    float amount = (1.0 - exp((-(at_eye) * _e65)));
    uint _e72 = layer_now;
    float4 _e75 = cameras[_e72].sun;
    uint _e84 = layer_now;
    float _e88 = cameras[_e84].fog.w;
    uint _e92 = layer_now;
    float _e96 = cameras[_e92].daylight.x;
    float sunward = ((pow(max(dot(d_2, _e75.xyz), 0.0), 8.0) * _e88) * _e96);
    const float3 _e101 = sky_tone(lerp(HORIZON, FOG_WARM, sunward), d_2);
    return lerp(colour_2, _e101, amount);
}

float3 to_srgb(float3 c)
{
    float3 low_1 = (c * 12.92);
    float3 high = ((1.055 * pow(c, (0.41666666).xxx)) - (0.055).xxx);
    return ((c <= (0.0031308).xxx) ? low_1 : high);
}

int naga_mod(int lhs, int rhs) {
    int divisor = ((lhs == int(-2147483647 - 1) & rhs == -1) | (rhs == 0)) ? 1 : rhs;
    return lhs - (lhs / divisor) * divisor;
}

typedef float3 ret_Constructarray6_float3_[6];
ret_Constructarray6_float3_ Constructarray6_float3_(float3 arg0, float3 arg1, float3 arg2, float3 arg3, float3 arg4, float3 arg5) {
    float3 ret[6] = { arg0, arg1, arg2, arg3, arg4, arg5 };
    return ret;
}

void finish(uint2 id_5, float3 d_3, float shown, Seen seen_2, bool joined_1, float chain)
{
    float3 here = (float3)0;
    float3 colour_3 = (float3)0;
    bool local_3 = (bool)0;
    bool local_4 = (bool)0;
    bool local_5 = (bool)0;
    bool local_6 = (bool)0;

    uint _e8 = layer_now;
    float4 _e11 = cameras[_e8].view;
    uint2 size_2 = naga_f2u32(_e11.xy);
    const float _e16 = worth_of(seen_2.distance_, seen_2.level);
    bool hidden = (round((_e16 * 255.0)) >= 255.0);
    here = seen_2.colour;
    if (!(hidden)) {
        const float3 _e25 = shaded(seen_2);
        here = _e25;
    }
    uint _e31 = layer_now;
    float _e35 = cameras[_e31].reach.x;
    float met = ((seen_2.distance_ >= _e35) ? seen_2.touched : seen_2.distance_);
    uint _e40 = layer_now;
    float _e44 = cameras[_e40].reach.x;
    if ((met < _e44)) {
        const float3 _e47 = sky(d_3);
        float3 behind = (seen_2.sky_share * _e47);
        float covered_1 = max((1.0 - seen_2.sky_share), 0.0001);
        float3 _e54 = here;
        const float3 _e58 = fogged(((_e54 - behind) / (covered_1).xxx), d_3, met);
        uint _e61 = layer_now;
        float _e65 = cameras[_e61].fade.w;
        float shown_2 = pow(covered_1, (1.0 + ((_e65 * seen_2.sky_share) * seen_2.sky_share)));
        const float3 _e73 = sky(d_3);
        here = ((_e73 * (1.0 - shown_2)) + (_e58 * shown_2));
    }
    uint _e81 = layer_now;
    float _e85 = cameras[_e81].layer.y;
    if ((_e85 > 0.5)) {
        const float _e95 = worth_of(seen_2.distance_, seen_2.level);
        put_worth(((id_5.y * size_2.x) + id_5.x), naga_f2u32(round((_e95 * 255.0))));
    }
    float3 _e100 = here;
    colour_3 = _e100;
    if (false) {
        uint _e107 = layer_now;
        float _e111 = cameras[_e107].reach.z;
        local_3 = (_e111 > 1.5);
    } else {
        local_3 = false;
    }
    bool _e115 = local_3;
    if (_e115) {
        local_4 = (seen_2.level >= int(0));
    } else {
        local_4 = false;
    }
    bool _e122 = local_4;
    if (_e122) {
        float3 tints[6] = Constructarray6_float3_(float3(1.0, 0.3, 0.3), float3(0.3, 1.0, 0.3), float3(0.3, 0.4, 1.0), float3(1.0, 1.0, 0.3), float3(1.0, 0.3, 1.0), float3(0.3, 1.0, 1.0));
        float3 _e148 = colour_3;
        colour_3 = ((_e148 * 0.4) + (tints[min(uint(naga_mod(seen_2.level, int(6))), 5u)] * 0.35));
    } else {
        if (false) {
            uint _e163 = layer_now;
            float _e167 = cameras[_e163].reach.z;
            local_5 = (_e167 > 0.5);
        } else {
            local_5 = false;
        }
        bool _e171 = local_5;
        if (_e171) {
            uint _e176 = layer_now;
            float _e180 = cameras[_e176].reach.z;
            local_6 = (_e180 < 1.5);
        } else {
            local_6 = false;
        }
        bool _e184 = local_6;
        if (_e184) {
            float k_9 = clamp((seen_2.steps / 256.0), 0.0, 1.0);
            colour_3 = ((k_9 < 0.5) ? lerp(float3(0.0, 0.0, 0.4), float3(0.0, 0.8, 0.2), (k_9 * 2.0)) : lerp(float3(0.0, 0.8, 0.2), float3(1.0, 0.0, 0.0), ((k_9 - 0.5) * 2.0)));
            float3 _e218 = colour_3;
            float3 _e219 = colour_3;
            colour_3 = (_e218 * _e219);
        }
    }
    float3 _e223 = colour_3;
    const float3 _e229 = to_srgb(clamp(_e223, (0.0).xxx, (1.0).xxx));
    put_picture(int2(id_5.xy), float4((_e229 * shown), shown));
    const float _e237 = list_long(id_5, seen_2.steps, joined_1);
    put_far(int2(id_5.xy), float4(seen_2.touched, seen_2.steps, chain, _e237));
    return;
}

float leaves(float2 o, float2 d_4, float2 lo, float2 hi)
{
    float2 to = ((d_4 > (0.0).xx) ? hi : lo);
    float2 t_3 = ((to - o) / ((abs(d_4) < (1e-9).xx) ? (1e-9).xx : d_4));
    return min(((abs(d_4.x) < 1e-9) ? 1e30 : t_3.x), ((abs(d_4.y) < 1e-9) ? 1e30 : t_3.y));
}

float map_end(float3 o_1, float3 d_5)
{
    bool local_7 = (bool)0;

    const int _e5 = table(REGION, int(1));
    const int _e9 = table(REGION, int(2));
    float2 lo_1 = (float2(float(_e5), float(_e9)) * 1024.0);
    const int _e15 = table(REGION, int(3));
    float2 hi_1 = (lo_1 + ((float(_e15) * 1024.0)).xx);
    if (!(any((o_1.xy < lo_1)))) {
        local_7 = any((o_1.xy > hi_1));
    } else {
        local_7 = true;
    }
    bool _e30 = local_7;
    if (_e30) {
        uint _e33 = layer_now;
        float _e37 = cameras[_e33].reach.x;
        return _e37;
    }
    const float _e40 = leaves(o_1.xy, d_5.xy, lo_1, hi_1);
    const int _e41 = level_at(_e40);
    float s = (2.0 * float((int(1) << uint(min(asint(asuint(_e41) + asuint(int(1))), TOP)))));
    const float _e58 = leaves(o_1.xy, d_5.xy, (lo_1 - (s).xx), (hi_1 + (s).xx));
    return _e58;
}

float weave(uint2 at_7, uint bits)
{
    uint v_3 = 0u;
    uint b = 0u;

    uint x = (at_7.x ^ at_7.y);
    uint2 loop_bound = uint2(4294967295u, 4294967295u);
    bool loop_init = true;
    while(true) {
        if (all(loop_bound == uint2(0u, 0u))) { break; }
        loop_bound -= uint2(loop_bound.y == 0u, 1u);
        if (!loop_init) {
            uint _e28 = b;
            b = (_e28 + 1u);
        }
        loop_init = false;
        uint _e9 = b;
        if ((_e9 < bits)) {
        } else {
            break;
        }
        {
            uint _e11 = v_3;
            uint _e14 = b;
            uint _e22 = b;
            v_3 = (((_e11 << 2u) | (((x >> _e14) & 1u) << 1u)) | ((at_7.y >> _e22) & 1u));
        }
    }
    uint _e30 = v_3;
    return ((float(_e30) + 0.5) / float((1u << (2u * bits))));
}

uint2 naga_mod(uint2 lhs, uint2 rhs) {
    return lhs % (rhs == 0u ? 1u : rhs);
}

float blue(uint2 at_8)
{
    uint2 side = NagaDimensions2D(t_blue);
    float4 _e7 = t_blue.Load(int3(int2(naga_mod(at_8, side)), int(0)));
    return (((_e7.x * 255.0) + 0.5) / 256.0);
}

float ray_start(uint2 at_9)
{
    float phase = (float)0;
    bool local_8 = (bool)0;

    uint _e3 = layer_now;
    float _e7 = cameras[_e3].ray_start.x;
    uint pattern = naga_f2u32(_e7);
    const float _e11 = scatter((at_9 * 2u));
    phase = _e11;
    if (!((pattern == 1u))) {
        local_8 = (pattern == 3u);
    } else {
        local_8 = true;
    }
    bool _e21 = local_8;
    if (_e21) {
        uint _e24 = layer_now;
        float _e28 = cameras[_e24].ray_start.y;
        const float _e30 = weave(at_9, naga_f2u32(_e28));
        phase = _e30;
        if ((pattern == 3u)) {
            float _e33 = phase;
            uint _e36 = layer_now;
            float _e40 = cameras[_e36].ray_start.z;
            const float _e41 = blue(at_9);
            phase = frac((_e33 + (_e40 * _e41)));
        }
    } else {
        if ((pattern == 2u)) {
            const float _e47 = blue(at_9);
            phase = _e47;
        }
    }
    float _e48 = phase;
    uint _e51 = layer_now;
    float _e55 = cameras[_e51].ray_start.w;
    return (_e48 * _e55);
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

float in_ring(int level_3, float3 p_2)
{
    uint _e4 = layer_now;
    float4 box = cameras[_e4].rings[asint(asuint(int(4)) - asuint(level_3))];
    return min(min((p_2.x - box.x), (box.z - p_2.x)), min((p_2.y - box.y), (box.w - p_2.y)));
}

int ring_level(int level_4, float3 p_3)
{
    int l = (int)0;
    bool local_9 = (bool)0;

    l = level_4;
    uint2 loop_bound_1 = uint2(4294967295u, 4294967295u);
    while(true) {
        if (all(loop_bound_1 == uint2(0u, 0u))) { break; }
        loop_bound_1 -= uint2(loop_bound_1.y == 0u, 1u);
        int _e3 = l;
        if ((_e3 < FIRST)) {
            int _e8 = l;
            const float _e9 = in_ring(_e8, p_3);
            local_9 = (_e9 < 0.0);
        } else {
            local_9 = false;
        }
        bool _e13 = local_9;
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

uint entry_at(int level_5, int cx, int cy)
{
    bool local_10 = (bool)0;
    bool local_11 = (bool)0;
    bool local_12 = (bool)0;

    const int _e4 = table(level_5, int(1));
    int x_1 = asint(asuint(cx) - asuint(_e4));
    const int _e7 = table(level_5, int(2));
    int y = asint(asuint(cy) - asuint(_e7));
    const int _e10 = table(level_5, int(3));
    if (!((x_1 < int(0)))) {
        local_10 = (y < int(0));
    } else {
        local_10 = true;
    }
    bool _e19 = local_10;
    if (!(_e19)) {
        local_11 = (x_1 >= _e10);
    } else {
        local_11 = true;
    }
    bool _e25 = local_11;
    if (!(_e25)) {
        local_12 = (y >= _e10);
    } else {
        local_12 = true;
    }
    bool _e31 = local_12;
    if (_e31) {
        return PAST_MAP;
    }
    const int _e35 = table(level_5, int(4));
    uint _e41 = asuint(entries.Load(uint(asint(asuint(asint(asuint(_e35) + asuint(asint(asuint(y) * asuint(_e10))))) + asuint(x_1)))*4));
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

void keep_place(inout Place place_2, float3 p_4)
{
    bool local_13 = (bool)0;
    uint usable = 0u;
    bool local_14 = (bool)0;
    bool local_15 = (bool)0;
    bool local_16 = (bool)0;
    int k_1 = int(0);
    bool local_17 = (bool)0;

    int2 r = naga_f2i32(floor((p_4.xy / (1024.0).xx)));
    bool _e9 = place_2.known;
    if (_e9) {
        int2 _e13 = place_2.region;
        local_13 = all((r == _e13));
    } else {
        local_13 = false;
    }
    bool _e17 = local_13;
    if (_e17) {
        return;
    }
    uint _e22 = layer_now;
    float _e26 = cameras[_e22].window.y;
    int n_3 = naga_f2i32(_e26);
    uint _e30 = layer_now;
    float4 _e33 = cameras[_e30].window;
    int2 rr = asint(asuint(r) - asuint(naga_f2i32(_e33.zw)));
    if ((rr.x >= int(0))) {
        local_14 = (rr.y >= int(0));
    } else {
        local_14 = false;
    }
    bool _e46 = local_14;
    if (_e46) {
        local_15 = (rr.x < n_3);
    } else {
        local_15 = false;
    }
    bool _e52 = local_15;
    if (_e52) {
        local_16 = (rr.y < n_3);
    } else {
        local_16 = false;
    }
    bool on_map = local_16;
    const int _e61 = table(TOP, int(4));
    const int _e64 = table(TOP, int(3));
    const int _e67 = table(TOP, int(3));
    int flags_at = asint(asuint(_e61) + asuint(asint(asuint(_e64) * asuint(_e67))));
    uint2 loop_bound_2 = uint2(4294967295u, 4294967295u);
    bool loop_init_1 = true;
    while(true) {
        if (all(loop_bound_2 == uint2(0u, 0u))) { break; }
        loop_bound_2 -= uint2(loop_bound_2.y == 0u, 1u);
        if (!loop_init_1) {
            int _e109 = k_1;
            k_1 = asint(asuint(_e109) + asuint(int(1)));
        }
        loop_init_1 = false;
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
                uint _e97 = asuint(entries.Load(uint(asint(asuint(asint(asuint(flags_at) + asuint(asint(asuint(asint(asuint(asint(asuint(_e87) * asuint(n_3))) + asuint(rr.y))) * asuint(n_3))))) + asuint(rr.x)))*4));
                local_17 = (_e97 != 0u);
            } else {
                local_17 = true;
            }
            bool _e101 = local_17;
            if (_e101) {
                uint _e102 = usable;
                int _e104 = k_1;
                usable = (_e102 | (1u << uint(_e104)));
            }
        }
    }
    const uint _e115 = entry_at(REGION, r.x, r.y);
    uint _e118 = usable;
    place_2 = ConstructPlace(r, true, (_e115 != NOT_SUMMED), _e118);
    return;
}

float leave2_(float2 o_2, float2 d_6, float2 c_1, float size)
{
    float2 next_1 = ((c_1 + ((d_6 > (0.0).xx) ? (1.0).xx : (0.0).xx)) * size);
    float2 t_4 = ((next_1 - o_2) / d_6);
    float tx = ((abs(d_6.x) > 1e-6) ? t_4.x : 1e30);
    float ty = ((abs(d_6.y) > 1e-6) ? t_4.y : 1e30);
    return min(tx, ty);
}

void keep_row(inout Row row, int level_6)
{
    int _e3 = row.level;
    if ((_e3 != level_6)) {
        const int _e6 = table(level_6, int(1));
        const int _e8 = table(level_6, int(2));
        const int _e11 = table(level_6, int(3));
        const int _e13 = table(level_6, int(4));
        row = ConstructRow(level_6, int2(_e6, _e8), _e11, _e13);
        return;
    } else {
        return;
    }
}

uint top_in(Row row_1, int cx_1, int cy_1)
{
    bool local_18 = (bool)0;
    bool local_19 = (bool)0;
    bool local_20 = (bool)0;

    int x_2 = asint(asuint(cx_1) - asuint(row_1.least.x));
    int y_1 = asint(asuint(cy_1) - asuint(row_1.least.y));
    if (!((x_2 < int(0)))) {
        local_18 = (y_1 < int(0));
    } else {
        local_18 = true;
    }
    bool _e17 = local_18;
    if (!(_e17)) {
        local_19 = (x_2 >= row_1.across);
    } else {
        local_19 = true;
    }
    bool _e24 = local_19;
    if (!(_e24)) {
        local_20 = (y_1 >= row_1.across);
    } else {
        local_20 = true;
    }
    bool _e31 = local_20;
    if (_e31) {
        return 4294967295u;
    }
    uint _e42 = layer_now;
    float _e46 = cameras[_e42].pyramid.w;
    uint _e50 = asuint(entries.Load((uint(asint(asuint(asint(asuint(row_1.entries_at) + asuint(asint(asuint(y_1) * asuint(row_1.across))))) + asuint(x_2))) + naga_f2u32(_e46))*4));
    return _e50;
}

void keep_level(inout Kept k_2, int level_7)
{
    int _e4 = k_2.row.level;
    if ((_e4 != level_7)) {
        const int _e7 = table(level_7, int(1));
        const int _e9 = table(level_7, int(2));
        const int _e12 = table(level_7, int(3));
        const int _e14 = table(level_7, int(4));
        const int _e17 = table(level_7, int(5));
        const int _e19 = table(level_7, int(6));
        const int _e22 = table(level_7, int(7));
        const int _e25 = table(level_7, int(8));
        const int _e27 = table(level_7, int(9));
        const int _e29 = table(level_7, int(10));
        k_2 = ConstructKept(ConstructRow(level_7, int2(_e7, _e9), _e12, _e14), _e17, uint(_e19), uint(_e22), _e25, _e27, _e29, (int(-1)).xxx, NO_SLOT);
        return;
    } else {
        return;
    }
}

uint entry_in(Row row_2, int cx_2, int cy_2)
{
    bool local_21 = (bool)0;
    bool local_22 = (bool)0;
    bool local_23 = (bool)0;

    int x_3 = asint(asuint(cx_2) - asuint(row_2.least.x));
    int y_2 = asint(asuint(cy_2) - asuint(row_2.least.y));
    if (!((x_3 < int(0)))) {
        local_21 = (y_2 < int(0));
    } else {
        local_21 = true;
    }
    bool _e17 = local_21;
    if (!(_e17)) {
        local_22 = (x_3 >= row_2.across);
    } else {
        local_22 = true;
    }
    bool _e24 = local_22;
    if (!(_e24)) {
        local_23 = (y_2 >= row_2.across);
    } else {
        local_23 = true;
    }
    bool _e31 = local_23;
    if (_e31) {
        return PAST_MAP;
    }
    uint _e41 = asuint(entries.Load(uint(asint(asuint(asint(asuint(row_2.entries_at) + asuint(asint(asuint(y_2) * asuint(row_2.across))))) + asuint(x_3)))*4));
    return _e41;
}

float rock_share(Row row_3, int3 base, float3 f)
{
    float solid = 0.0;
    int n = int(0);
    bool local_24 = (bool)0;
    bool local_25 = (bool)0;
    bool local_26 = (bool)0;
    bool local_27 = (bool)0;
    bool local_28 = (bool)0;

    uint2 loop_bound_3 = uint2(4294967295u, 4294967295u);
    bool loop_init_2 = true;
    while(true) {
        if (all(loop_bound_3 == uint2(0u, 0u))) { break; }
        loop_bound_3 -= uint2(loop_bound_3.y == 0u, 1u);
        if (!loop_init_2) {
            int _e100 = n;
            n = asint(asuint(_e100) + asuint(int(1)));
        }
        loop_init_2 = false;
        int _e7 = n;
        if ((_e7 < int(4))) {
        } else {
            break;
        }
        {
            int _e11 = n;
            int _e16 = n;
            const uint _e20 = entry_in(row_3, asint(asuint(base.x) + asuint((_e11 & int(1)))), asint(asuint(base.y) + asuint((_e16 >> 1u))));
            if (!((_e20 == NOT_SUMMED))) {
                local_24 = (_e20 == PAST_MAP);
            } else {
                local_24 = true;
            }
            bool _e29 = local_24;
            if (_e29) {
                continue;
            }
            int low_2 = (asint((_e20 << 16u)) >> 16u);
            if (!((base.z < int(0)))) {
                if ((base.z < low_2)) {
                    local_26 = (low_2 > int(0));
                } else {
                    local_26 = false;
                }
                bool _e48 = local_26;
                local_25 = _e48;
            } else {
                local_25 = true;
            }
            bool _e50 = local_25;
            float lower = float(_e50);
            if (!((asint(asuint(base.z) + asuint(int(1))) < int(0)))) {
                if ((asint(asuint(base.z) + asuint(int(1))) < low_2)) {
                    local_28 = (low_2 > int(0));
                } else {
                    local_28 = false;
                }
                bool _e69 = local_28;
                local_27 = _e69;
            } else {
                local_27 = true;
            }
            bool _e71 = local_27;
            float upper = float(_e71);
            int _e77 = n;
            int _e87 = n;
            float w_2 = ((((_e77 & int(1)) == int(1)) ? f.x : (1.0 - f.x)) * (((_e87 >> 1u) == int(1)) ? f.y : (1.0 - f.y)));
            float _e94 = solid;
            solid = (_e94 + (w_2 * lerp(lower, upper, f.z)));
        }
    }
    float _e102 = solid;
    return _e102;
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

Spot spot_soft(inout Kept k_3, Place place_3, int level_8, float3 p_5)
{
    bool inside = false;
    bool local_29 = (bool)0;
    bool local_30 = (bool)0;
    int n_1 = int(0);
    bool local_31 = (bool)0;
    bool local_32 = (bool)0;
    bool local_33 = (bool)0;
    bool local_34 = (bool)0;
    bool local_35 = (bool)0;
    bool local_36 = (bool)0;
    bool local_37 = (bool)0;
    bool local_38 = (bool)0;
    bool local_39 = (bool)0;
    bool local_40 = (bool)0;
    bool local_41 = (bool)0;

    float size_3 = float((int(1) << uint(level_8)));
    float3 u = ((p_5 / (size_3).xxx) - (0.5).xxx);
    float3 b_2 = floor(u);
    float3 f_2 = (u - b_2);
    int3 base_1 = naga_f2i32(b_2);
    Row row_6 = k_3.row;
    if ((level_8 <= REGION)) {
        local_29 = place_3.summed;
    } else {
        local_29 = false;
    }
    bool _e26 = local_29;
    if (_e26) {
        uint2 shift_1 = (uint(asint(asuint(REGION) - asuint(level_8)))).xx;
        if (all(((base_1.xy >> shift_1) == place_3.region))) {
            local_30 = all(((asint(asuint(base_1.xy) + asuint((int(1)).xx)) >> shift_1) == place_3.region));
        } else {
            local_30 = false;
        }
        bool _e47 = local_30;
        inside = _e47;
    }
    bool _e48 = inside;
    if (!(_e48)) {
        uint2 loop_bound_4 = uint2(4294967295u, 4294967295u);
        bool loop_init_3 = true;
        while(true) {
            if (all(loop_bound_4 == uint2(0u, 0u))) { break; }
            loop_bound_4 -= uint2(loop_bound_4.y == 0u, 1u);
            if (!loop_init_3) {
                int _e73 = n_1;
                n_1 = asint(asuint(_e73) + asuint(int(1)));
            }
            loop_init_3 = false;
            int _e52 = n_1;
            if ((_e52 < int(4))) {
            } else {
                break;
            }
            {
                int _e56 = n_1;
                int _e61 = n_1;
                const uint _e65 = entry_in(row_6, asint(asuint(base_1.x) + asuint((_e56 & int(1)))), asint(asuint(base_1.y) + asuint((_e61 >> 1u))));
                if ((_e65 == NOT_SUMMED)) {
                    const Spot spot_1 = ConstructSpot(int(0), level_8, (0.0).xxx);
                    return spot_1;
                }
            }
        }
    }
    int x_4 = asint(asuint(base_1.x) - asuint(row_6.least.x));
    int y_3 = asint(asuint(base_1.y) - asuint(row_6.least.y));
    if (LOADED) {
        local_31 = (base_1.z < int(0));
    } else {
        local_31 = false;
    }
    bool under = local_31;
    if (LOADED) {
        local_32 = (x_4 == int(-1));
    } else {
        local_32 = false;
    }
    bool _e97 = local_32;
    if (LOADED) {
        local_33 = (y_3 == int(-1));
    } else {
        local_33 = false;
    }
    bool _e104 = local_33;
    bool2 short_ = bool2(_e97, _e104);
    if ((x_4 < int(0))) {
        local_34 = !(short_.x);
    } else {
        local_34 = false;
    }
    bool _e113 = local_34;
    if (!(_e113)) {
        if ((y_3 < int(0))) {
            local_36 = !(short_.y);
        } else {
            local_36 = false;
        }
        bool _e124 = local_36;
        local_35 = _e124;
    } else {
        local_35 = true;
    }
    bool _e126 = local_35;
    if (!(_e126)) {
        if ((base_1.z < int(0))) {
            local_38 = !(under);
        } else {
            local_38 = false;
        }
        bool _e137 = local_38;
        local_37 = _e137;
    } else {
        local_37 = true;
    }
    bool _e139 = local_37;
    if (!(_e139)) {
        local_39 = (x_4 >= row_6.across);
    } else {
        local_39 = true;
    }
    bool _e146 = local_39;
    if (!(_e146)) {
        local_40 = (y_3 >= row_6.across);
    } else {
        local_40 = true;
    }
    bool _e153 = local_40;
    if (!(_e153)) {
        int _e159 = k_3.up;
        local_41 = (base_1.z >= _e159);
    } else {
        local_41 = true;
    }
    bool _e162 = local_41;
    if (_e162) {
        if (LOADED) {
            const float _e164 = rock_share(row_6, base_1, f_2);
            const Spot spot_2 = ConstructSpot(int(5), level_8, float3(_e164, 0.0, 0.0));
            return spot_2;
        }
        const Spot spot_3 = ConstructSpot(int(1), level_8, (0.0).xxx);
        return spot_3;
    }
    uint wide = k_3.wide;
    uint tall = k_3.tall;
    int cx_5 = max(x_4, int(0));
    int cy_5 = max(y_3, int(0));
    int cz_2 = max(base_1.z, int(0));
    int3 brick = int3(int((uint(cx_5) >> wide)), int((uint(cy_5) >> wide)), int((uint(cz_2) >> tall)));
    int3 _e196 = k_3.brick;
    if (any((brick != _e196))) {
        k_3.brick = brick;
        int _e203 = k_3.index_at;
        int _e206 = k_3.bricks;
        int _e211 = k_3.bricks_up;
        uint _e218 = asuint(slots.Load(uint(asint(asuint(asint(asuint(_e203) + asuint(asint(asuint(asint(asuint(asint(asuint(brick.y) * asuint(_e206))) + asuint(brick.x))) * asuint(_e211))))) + asuint(brick.z)))*4));
        k_3.slot = _e218;
    }
    uint slot_7 = k_3.slot;
    if ((slot_7 == NO_SLOT)) {
        if (LOADED) {
            const float _e224 = rock_share(row_6, base_1, f_2);
            const Spot spot_4 = ConstructSpot(int(5), level_8, float3(_e224, 0.0, 0.0));
            return spot_4;
        }
        const Spot spot_5 = ConstructSpot(int(1), level_8, (0.0).xxx);
        return spot_5;
    }
    uint _e248 = layer_now;
    float _e252 = cameras[_e248].atlas.w;
    float3 origin = (float3(float((slot_7 & 255u)), float(((slot_7 >> 8u) & 255u)), float((slot_7 >> 16u))) * _e252);
    float3 local_68 = float3(float((uint(cx_5) & ((1u << wide) - 1u))), float((uint(cy_5) & ((1u << wide) - 1u))), float((uint(cz_2) & ((1u << tall) - 1u))));
    float3 within = float3((short_.x ? 0.0 : f_2.x), (short_.y ? 0.0 : f_2.y), (under ? 0.0 : f_2.z));
    int kind = asint(asuint(asint(asuint(asint(asuint(int(2)) + asuint((under ? int(8) : int(0))))) + asuint((short_.x ? int(16) : int(0))))) + asuint((short_.y ? int(32) : int(0))));
    uint _e310 = layer_now;
    float4 _e313 = cameras[_e310].atlas;
    const Spot spot_6 = ConstructSpot(kind, level_8, ((((origin + local_68) + within) + (0.5).xxx) / _e313.xyz));
    return spot_6;
}

Spot spot_kept(inout Kept k_4, Place place_4, int level_9, float3 p_6)
{
    int l_1 = (int)0;
    bool local_42 = (bool)0;
    bool local_43 = (bool)0;
    bool local_44 = (bool)0;

    l_1 = level_9;
    int _e5 = l_1;
    const int _e6 = ring_level(_e5, p_6);
    l_1 = _e6;
    uint2 loop_bound_5 = uint2(4294967295u, 4294967295u);
    while(true) {
        if (all(loop_bound_5 == uint2(0u, 0u))) { break; }
        loop_bound_5 -= uint2(loop_bound_5.y == 0u, 1u);
        int _e7 = l_1;
        if ((_e7 >= FIRST)) {
            int _e12 = l_1;
            uint _e17 = layer_now;
            float _e21 = cameras[_e17].window.x;
            local_42 = (asint(asuint(_e12) - asuint(FIRST)) < naga_f2i32(_e21));
        } else {
            local_42 = false;
        }
        bool _e25 = local_42;
        if (_e25) {
            int _e30 = l_1;
            local_43 = ((place_4.usable & (1u << uint(asint(asuint(_e30) - asuint(FIRST))))) == 0u);
        } else {
            local_43 = false;
        }
        bool _e39 = local_43;
        if (_e39) {
        } else {
            break;
        }
        {
            int _e40 = l_1;
            l_1 = asint(asuint(_e40) + asuint(int(1)));
        }
    }
    if (false) {
        uint _e48 = layer_now;
        float _e52 = cameras[_e48].layer.w;
        local_44 = (_e52 < 0.5);
    } else {
        local_44 = false;
    }
    bool _e56 = local_44;
    if (_e56) {
        int _e57 = l_1;
        const Spot spot_7 = ConstructSpot(int(1), _e57, (0.0).xxx);
        return spot_7;
    }
    int _e62 = l_1;
    keep_level(k_4, _e62);
    int _e63 = l_1;
    const Spot _e64 = spot_soft(k_4, place_4, _e63, p_6);
    const Spot spot_8 = _e64;
    return spot_8;
}

Fetched ConstructFetched(float2 arg0, float4 arg1, float3 arg2, float3 arg3) {
    Fetched ret = (Fetched)0;
    ret.sw = arg0;
    ret.c = arg1;
    ret.plus = arg2;
    ret.minus = arg3;
    return ret;
}

Fetched fetch_together(float3 at_10)
{
    float4 _e4 = t_solid.SampleLevel(soft, at_10, 0.0);
    float4 _e9 = t_colour.SampleLevel(soft, at_10, 0.0);
    float4 _e13 = t_plus.SampleLevel(soft, at_10, 0.0);
    float4 _e18 = t_minus.SampleLevel(soft, at_10, 0.0);
    const Fetched fetched = ConstructFetched(_e4.xy, _e9, _e13.xyz, _e18.xyz);
    return fetched;
}

Look empty_look(bool known)
{
    Look look = (Look)0;

    look.known = known;
    Look _e3 = look;
    const Look look_9 = _e3;
    return look_9;
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
    const Look look_10 = _e31;
    return look_10;
}

Look mix_looks(Look a, Look b_1, float w)
{
    Look out_ = (Look)0;

    float k_10 = clamp(w, 0.0, 1.0);
    out_ = a;
    out_.solid = lerp(a.solid, b_1.solid, k_10);
    out_.water = lerp(a.water, b_1.water, k_10);
    out_.colour = lerp(a.colour, b_1.colour, k_10);
    out_.plus = lerp(a.plus, b_1.plus, k_10);
    out_.minus = lerp(a.minus, b_1.minus, k_10);
    out_.surface = lerp(a.surface, b_1.surface, k_10);
    Look _e31 = out_;
    const Look look_11 = _e31;
    return look_11;
}

float2 faces_light(Look look_2, float3 d_7)
{
    float light = 0.0;

    uint _e4 = layer_now;
    float4 _e7 = cameras[_e4].sun;
    float3 sun_1 = _e7.xyz;
    float3 seen_plus = (look_2.plus * max(-(d_7), (0.0).xxx));
    float3 seen_minus = (look_2.minus * max(d_7, (0.0).xxx));
    float seen_4 = dot((seen_plus + seen_minus), (1.0).xxx);
    float3 sunny_plus = max(sun_1, (0.0).xxx);
    float3 sunny_minus = max(-(sun_1), (0.0).xxx);
    if ((seen_4 > 0.0001)) {
        light = ((dot(seen_plus, sunny_plus) + dot(seen_minus, sunny_minus)) / seen_4);
    } else {
        light = (dot(look_2.plus, sunny_plus) + dot(look_2.minus, sunny_minus));
    }
    float _e44 = light;
    return float2(_e44, seen_4);
}

Look by_finer_cell(Look look_3, int up, float3 d_8)
{
    Look out_1 = (Look)0;

    out_1 = look_3;
    if ((up > int(0))) {
        float finer = exp2(-(float(up)));
        out_1.solid = (1.0 - pow((1.0 - min(look_3.solid, 0.9999)), finer));
        if (FACES) {
            const float2 _e20 = faces_light(look_3, d_8);
            float shown_3 = min((look_3.surface * _e20.y), 0.9999);
            out_1.surface = ((look_3.surface * (1.0 - pow((1.0 - shown_3), finer))) / max(shown_3, 1e-6));
        }
    }
    Look _e36 = out_1;
    const Look look_12 = _e36;
    return look_12;
}

Look solid_look()
{
    Look look_4 = (Look)0;

    look_4.known = true;
    look_4.solid = 1.0;
    Look _e5 = look_4;
    const Look look_13 = _e5;
    return look_13;
}

Look cell_of(uint4 w_1)
{
    Look look_5 = (Look)0;

    look_5.known = true;
    look_5.solid = (float((w_1.x & 255u)) / 255.0);
    look_5.water = (float(((w_1.x >> 8u) & 255u)) / 255.0);
    float up_2 = float(((w_1.x >> 16u) & 255u));
    float down = float((w_1.x >> 24u));
    float north = float((w_1.y & 255u));
    float south = float(((w_1.y >> 8u) & 255u));
    float east = float(((w_1.y >> 16u) & 255u));
    float west = max((((((255.0 - up_2) - down) - north) - south) - east), 0.0);
    look_5.plus = (float3(north, east, up_2) / (255.0).xxx);
    look_5.minus = (float3(south, west, down) / (255.0).xxx);
    look_5.colour = (float3(float(((w_1.z >> 20u) & 1023u)), float(((w_1.z >> 10u) & 1023u)), float((w_1.z & 1023u))) / (1023.0).xxx);
    look_5.surface = min((float((w_1.w >> 16u)) / 256.0), 4.0);
    Look _e94 = look_5;
    const Look look_14 = _e94;
    return look_14;
}

uint4 cell_word(int level_10, int cx_3, int cy_3, int cz)
{
    const int _e5 = table(level_10, int(6));
    uint wide_1 = uint(_e5);
    const int _e8 = table(level_10, int(7));
    uint tall_1 = uint(_e8);
    const int _e11 = table(level_10, int(1));
    uint x_5 = uint(asint(asuint(cx_3) - asuint(_e11)));
    const int _e15 = table(level_10, int(2));
    uint y_4 = uint(asint(asuint(cy_3) - asuint(_e15)));
    uint z = uint(cz);
    const int _e22 = table(level_10, int(8));
    const int _e28 = table(level_10, int(9));
    int brick_1 = asint(asuint(asint(asuint(asint(asuint(asint(asuint(int((y_4 >> wide_1))) * asuint(_e22))) + asuint(int((x_5 >> wide_1))))) * asuint(_e28))) + asuint(int((z >> tall_1))));
    const int _e35 = table(level_10, int(5));
    uint start = asuint(index.Load(uint(asint(asuint(_e35) + asuint(brick_1)))*4));
    if ((start == NO_BRICK)) {
        return (0u).xxxx;
    }
    uint mask = ((1u << wide_1) - 1u);
    uint4 _e64 = asuint(pool.Load4((((start + (x_5 & mask)) + ((y_4 & mask) << wide_1)) + ((z & ((1u << tall_1) - 1u)) << (2u * wide_1)))*16));
    return _e64;
}

Look from_band(int level_11, int cx_4, int cy_4, uint e, int cz_1)
{
    bool local_45 = (bool)0;
    bool local_46 = (bool)0;
    bool local_47 = (bool)0;

    if ((e == NOT_SUMMED)) {
        const Look _e8 = empty_look(false);
        const Look look_15 = _e8;
        return look_15;
    }
    if ((e == PAST_MAP)) {
        const Look _e12 = empty_look(true);
        const Look look_16 = _e12;
        return look_16;
    }
    int low_3 = (asint((e << 16u)) >> 16u);
    int count = int((e >> 16u));
    if (!((cz_1 < int(0)))) {
        if ((cz_1 < low_3)) {
            local_46 = (low_3 > int(0));
        } else {
            local_46 = false;
        }
        bool _e32 = local_46;
        local_45 = _e32;
    } else {
        local_45 = true;
    }
    bool _e34 = local_45;
    if (_e34) {
        const Look _e35 = solid_look();
        const Look look_17 = _e35;
        return look_17;
    }
    if (!((cz_1 < low_3))) {
        local_47 = (cz_1 >= asint(asuint(low_3) + asuint(count)));
    } else {
        local_47 = true;
    }
    bool _e43 = local_47;
    if (_e43) {
        const Look _e45 = empty_look(true);
        const Look look_18 = _e45;
        return look_18;
    }
    const uint4 _e46 = cell_word(level_11, cx_4, cy_4, cz_1);
    const Look _e47 = cell_of(_e46);
    const Look look_19 = _e47;
    return look_19;
}

Look blend(int level_12, float3 p_7)
{
    Look out_2 = (Look)0;
    float weight = 0.0;
    int k_5 = int(0);
    int oz = (int)0;

    float size_4 = float((int(1) << uint(level_12)));
    float3 u_1 = ((p_7 / (size_4).xxx) - (0.5).xxx);
    float3 b_3 = floor(u_1);
    float3 f_3 = (u_1 - b_3);
    int3 base_2 = naga_f2i32(b_3);
    const Look _e15 = empty_look(true);
    out_2 = _e15;
    uint2 loop_bound_6 = uint2(4294967295u, 4294967295u);
    bool loop_init_4 = true;
    while(true) {
        if (all(loop_bound_6 == uint2(0u, 0u))) { break; }
        loop_bound_6 -= uint2(loop_bound_6.y == 0u, 1u);
        if (!loop_init_4) {
            int _e105 = k_5;
            k_5 = asint(asuint(_e105) + asuint(int(1)));
        }
        loop_init_4 = false;
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
            int cx_6 = asint(asuint(base_2.x) + asuint(ox));
            int cy_6 = asint(asuint(base_2.y) + asuint(oy));
            const uint _e34 = entry_at(level_12, cx_6, cy_6);
            if ((_e34 == NOT_SUMMED)) {
                const Look _e38 = empty_look(false);
                const Look look_20 = _e38;
                return look_20;
            }
            float wxy = (((ox == int(1)) ? f_3.x : (1.0 - f_3.x)) * ((oy == int(1)) ? f_3.y : (1.0 - f_3.y)));
            oz = int(0);
            uint2 loop_bound_7 = uint2(4294967295u, 4294967295u);
            bool loop_init_5 = true;
            while(true) {
                if (all(loop_bound_7 == uint2(0u, 0u))) { break; }
                loop_bound_7 -= uint2(loop_bound_7.y == 0u, 1u);
                if (!loop_init_5) {
                    int _e102 = oz;
                    oz = asint(asuint(_e102) + asuint(int(1)));
                }
                loop_init_5 = false;
                int _e56 = oz;
                if ((_e56 < int(2))) {
                } else {
                    break;
                }
                {
                    int _e60 = oz;
                    const Look _e62 = from_band(level_12, cx_6, cy_6, _e34, asint(asuint(base_2.z) + asuint(_e60)));
                    int _e67 = oz;
                    float w_3 = (wxy * ((_e67 == int(1)) ? f_3.z : (1.0 - f_3.z)));
                    float _e73 = out_2.solid;
                    out_2.solid = (_e73 + (w_3 * _e62.solid));
                    float _e78 = out_2.water;
                    out_2.water = (_e78 + (w_3 * _e62.water));
                    float shown_4 = (w_3 * _e62.surface);
                    float3 _e85 = out_2.colour;
                    out_2.colour = (_e85 + (shown_4 * _e62.colour));
                    float3 _e90 = out_2.plus;
                    out_2.plus = (_e90 + (shown_4 * _e62.plus));
                    float3 _e95 = out_2.minus;
                    out_2.minus = (_e95 + (shown_4 * _e62.minus));
                    float _e99 = weight;
                    weight = (_e99 + shown_4);
                }
            }
        }
    }
    float _e107 = weight;
    if ((_e107 > 0.0)) {
        float3 _e111 = out_2.colour;
        float _e112 = weight;
        out_2.colour = (_e111 / (_e112).xxx);
        float3 _e116 = out_2.plus;
        float _e117 = weight;
        out_2.plus = (_e116 / (_e117).xxx);
        float3 _e121 = out_2.minus;
        float _e122 = weight;
        out_2.minus = (_e121 / (_e122).xxx);
        float _e126 = weight;
        out_2.surface = _e126;
    } else {
        out_2.colour = ROCK;
    }
    Look _e129 = out_2;
    const Look look_21 = _e129;
    return look_21;
}

Look look_at(Spot spot, float3 p_8)
{
    bool local_48 = (bool)0;
    Look inside_1 = (Look)0;
    Look look_6 = (Look)0;

    if ((spot.kind == int(0))) {
        const Look _e6 = empty_look(false);
        const Look look_22 = _e6;
        return look_22;
    }
    if (false) {
        local_48 = (spot.kind == int(1));
    } else {
        local_48 = false;
    }
    bool _e14 = local_48;
    if (_e14) {
        const Look _e16 = blend(spot.level, p_8);
        const Look look_23 = _e16;
        return look_23;
    }
    if ((spot.kind == int(5))) {
        const Look _e21 = empty_look(true);
        inside_1 = _e21;
        inside_1.solid = spot.at.x;
        inside_1.colour = ROCK;
        Look _e28 = inside_1;
        const Look look_24 = _e28;
        return look_24;
    }
    const Fetched _e30 = fetch_together(spot.at);
    const Look _e31 = look_of(_e30);
    look_6 = _e31;
    if ((spot.kind != int(2))) {
        float3 u_2 = ((p_8 / (float((int(1) << uint(spot.level)))).xxx) - (0.5).xxx);
        float3 f_4 = (u_2 - floor(u_2));
        if (((spot.kind & int(8)) != int(0))) {
            float lowest = clamp((u_2.z + 1.0), 0.0, 1.0);
            float _e61 = look_6.solid;
            look_6.solid = lerp(1.0, _e61, lowest);
            float _e65 = look_6.water;
            look_6.water = (_e65 * lowest);
            float _e68 = look_6.surface;
            look_6.surface = (_e68 * lowest);
        }
        float share = ((((spot.kind & int(16)) != int(0)) ? f_4.x : 1.0) * (((spot.kind & int(32)) != int(0)) ? f_4.y : 1.0));
        float _e88 = look_6.solid;
        look_6.solid = (_e88 * share);
        float _e91 = look_6.water;
        look_6.water = (_e91 * share);
        float _e94 = look_6.surface;
        look_6.surface = (_e94 * share);
    }
    Look _e96 = look_6;
    const Look look_25 = _e96;
    return look_25;
}

Look far_blend_kept(inout Kept k_6, Place place_5, int level_13, float3 p_9)
{
    const Spot _e4 = spot_kept(k_6, place_5, level_13, p_9);
    const Look _e5 = look_at(_e4, p_9);
    const Look look_26 = _e5;
    return look_26;
}

float shown_share(Look look_7, float seen_3)
{
    return ((look_7.surface * seen_3) * (1.0 - min((look_7.water * 64.0), 1.0)));
}

float3 lit_(float3 colour_4, float light_1)
{
    uint _e4 = layer_now;
    float _e8 = cameras[_e4].sun.w;
    uint _e11 = layer_now;
    float _e15 = cameras[_e11].daylight.y;
    uint _e19 = layer_now;
    float _e23 = cameras[_e19].sun.w;
    uint _e28 = layer_now;
    float _e32 = cameras[_e28].daylight.x;
    return (colour_4 * ((_e8 * _e15) + (((1.0 - _e23) * _e32) * light_1)));
}

float3 sunlit(float3 colour_5, float light_2)
{
    uint _e4 = layer_now;
    float _e8 = cameras[_e4].sun.w;
    uint _e14 = layer_now;
    float _e18 = cameras[_e14].daylight.x;
    return (((colour_5 * (1.0 - _e8)) * _e18) * light_2);
}

Seen ConstructSeen(float3 arg0, float arg1, float arg2, int arg3, float3 arg4, float3 arg5, float arg6, float3 arg7, float arg8, float arg9, float arg10, int arg11, bool arg12, float arg13) {
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
    ret.reached = arg10;
    ret.reached_level = arg11;
    ret.ended_in_air = arg12;
    ret.ended_at = arg13;
    return ret;
}

Look ZeroValueLook() {
    return (Look)0;
}

Seen march(float3 o_3, float3 d_9, float phase_1, float begin, float end, float stop, float through0_, bool look_first)
{
    float3 colour_6 = (0.0).xxx;
    float3 sunny = (0.0).xxx;
    float through = (float)0;
    float first_1 = (float)0;
    float touched = (float)0;
    int steps_1 = int(0);
    float t_2 = (float)0;
    bool in_air = (bool)0;
    float last_air = (float)0;
    bool walking = false;
    float walk_to = 0.0;
    int first_level_1 = int(-1);
    Look before = (Look)0;
    float before_t = 0.0;
    bool chained = false;
    bool floored = false;
    bool bottom = false;
    float3 bottom_here = (0.0).xxx;
    float3 bottom_sun = (0.0).xxx;
    float bottom_t = 0.0;
    int bottom_level = int(-1);
    Place place_6 = ConstructPlace((int(0)).xx, false, false, 0u);
    Kept kept = (Kept)0;
    Kept kept_next = (Kept)0;
    Row row_own = (Row)0;
    Row row_up = (Row)0;
    Row row_far = (Row)0;
    int lv = FIRST;
    float lv_from = 1.0;
    float lv_to = 0.0;
    float lv_hand = 0.0;
    int n_2 = int(0);
    bool local_49 = (bool)0;
    bool local_50 = (bool)0;
    bool local_51 = (bool)0;
    bool local_52 = (bool)0;
    bool local_53 = (bool)0;
    int level_14 = (int)0;
    bool skipped = (bool)0;
    uint high_far = (uint)0;
    uint high_up = (uint)0;
    uint high_own = (uint)0;
    bool local_54 = (bool)0;
    int k_7 = (int)0;
    bool local_55 = (bool)0;
    bool local_56 = (bool)0;
    float next = (float)0;
    float over = (float)0;
    bool local_57 = (bool)0;
    bool local_58 = (bool)0;
    float mixed = (float)0;
    Look look_8 = (Look)0;
    bool local_59 = (bool)0;
    bool local_60 = (bool)0;
    bool local_61 = (bool)0;
    float dt = (float)0;
    bool local_62 = (bool)0;
    bool local_63 = (bool)0;
    bool local_64 = (bool)0;
    float solid_1 = (float)0;
    float water = (float)0;
    float covered = (float)0;
    bool gathers = (bool)0;
    bool local_65 = (bool)0;

    through = through0_;
    uint _e17 = layer_now;
    float _e21 = cameras[_e17].reach.x;
    first_1 = _e21;
    uint _e25 = layer_now;
    float _e29 = cameras[_e25].reach.x;
    touched = _e29;
    t_2 = (max(begin, 0.5) + phase_1);
    in_air = !(look_first);
    float _e39 = t_2;
    last_air = _e39;
    const float _e52 = world_top();
    float ceiling = (_e52 + 32.0);
    float surface_step = (0.25 / clamp(abs(d_9.z), 0.25, 1.0));
    uint _e64 = layer_now;
    float4 _e67 = cameras[_e64].water;
    float3 water_colour = _e67.xyz;
    const Kept _e90 = kept_none();
    kept = _e90;
    const Kept _e92 = kept_none();
    kept_next = _e92;
    const Row _e94 = row_none();
    row_own = _e94;
    const Row _e96 = row_none();
    row_up = _e96;
    const Row _e98 = row_none();
    row_far = _e98;
    uint _e110 = layer_now;
    float _e114 = cameras[_e110].reach.x;
    float last = min(_e114, end);
    uint _e118 = layer_now;
    float _e122 = cameras[_e118].pyramid.z;
    int most = naga_f2i32(_e122);
    uint2 loop_bound_8 = uint2(4294967295u, 4294967295u);
    bool loop_init_6 = true;
    while(true) {
        if (all(loop_bound_8 == uint2(0u, 0u))) { break; }
        loop_bound_8 -= uint2(loop_bound_8.y == 0u, 1u);
        if (!loop_init_6) {
            int _e897 = n_2;
            n_2 = asint(asuint(_e897) + asuint(int(1)));
        }
        loop_init_6 = false;
        int _e126 = n_2;
        if ((_e126 < most)) {
        } else {
            break;
        }
        {
            float _e128 = t_2;
            if (!((_e128 > last))) {
                float _e133 = through;
                local_49 = (_e133 < 0.01);
            } else {
                local_49 = true;
            }
            bool _e137 = local_49;
            if (!(_e137)) {
                float _e141 = before_t;
                local_50 = (_e141 >= stop);
            } else {
                local_50 = true;
            }
            bool _e144 = local_50;
            if (_e144) {
                break;
            }
            int _e145 = n_2;
            steps_1 = asint(asuint(_e145) + asuint(int(1)));
            float _e148 = t_2;
            float3 p_10 = (o_3 + (d_9 * _e148));
            if ((p_10.z > ceiling)) {
                local_51 = (d_9.z >= 0.0);
            } else {
                local_51 = false;
            }
            bool _e159 = local_51;
            if (_e159) {
                break;
            }
            if ((p_10.z < -8.0)) {
                local_52 = (d_9.z <= 0.0);
            } else {
                local_52 = false;
            }
            bool _e169 = local_52;
            if (_e169) {
                floored = true;
                break;
            }
            float _e171 = t_2;
            float _e172 = lv_from;
            if (!((_e171 < _e172))) {
                float _e177 = t_2;
                float _e178 = lv_to;
                local_53 = (_e177 >= _e178);
            } else {
                local_53 = true;
            }
            bool _e181 = local_53;
            if (_e181) {
                float _e182 = t_2;
                const int _e183 = level_at(_e182);
                lv = _e183;
                float _e184 = t_2;
                uint _e187 = layer_now;
                float _e191 = cameras[_e187].view.z;
                float k_11 = floor(log2(max((_e184 * _e191), 1.0)));
                uint _e200 = layer_now;
                float _e204 = cameras[_e200].view.z;
                lv_from = ((k_11 < 0.5) ? 0.0 : ((exp2(k_11) / _e204) * 1.00001));
                uint _e217 = layer_now;
                float _e221 = cameras[_e217].view.z;
                lv_to = ((exp2((k_11 + 1.0)) / _e221) * 0.99999);
                int _e225 = lv;
                uint _e229 = layer_now;
                float _e233 = cameras[_e229].reach.y;
                uint _e237 = layer_now;
                float _e241 = cameras[_e237].fade.z;
                uint _e246 = layer_now;
                float _e250 = cameras[_e246].view.z;
                lv_hand = ((exp2(((float(_e225) - _e233) + _e241)) / _e250) * 0.99999);
            }
            int _e254 = lv;
            level_14 = _e254;
            int _e256 = level_14;
            const int _e257 = ring_level(_e256, p_10);
            level_14 = _e257;
            keep_place(place_6, p_10);
            int _e258 = level_14;
            if ((_e258 > REGION)) {
                int _e262 = level_14;
                float2 c_3 = floor((p_10.xy / (float((int(1) << uint(_e262)))).xx));
                int _e270 = level_14;
                const uint _e275 = entry_at(_e270, naga_f2i32(c_3.x), naga_f2i32(c_3.y));
                if ((_e275 == NOT_SUMMED)) {
                    level_14 = REGION;
                }
            }
            int _e279 = level_14;
            if ((_e279 <= REGION)) {
                bool _e283 = place_6.summed;
                if (!(_e283)) {
                    int2 _e288 = place_6.region;
                    const float _e291 = leave2_(o_3.xy, d_9.xy, float2(_e288), 1024.0);
                    float _e292 = t_2;
                    t_2 = (max(_e291, _e292) + 0.01);
                    float _e296 = t_2;
                    last_air = _e296;
                    walking = false;
                    chained = false;
                    float _e299 = t_2;
                    before_t = _e299;
                    continue;
                }
            }
            skipped = false;
            int _e302 = level_14;
            float fine = float((int(1) << uint(_e302)));
            high_far = 0u;
            high_up = 0u;
            high_own = 0u;
            bool _e313 = in_air;
            if (_e313) {
                bool _e316 = walking;
                local_54 = !(_e316);
            } else {
                local_54 = false;
            }
            bool _e319 = local_54;
            if (_e319) {
                int _e320 = level_14;
                int far_level = min(asint(asuint(_e320) + asuint(int(4))), TOP);
                int _e325 = level_14;
                int up_level = min(asint(asuint(_e325) + asuint(int(2))), TOP);
                keep_row(row_far, far_level);
                keep_row(row_up, up_level);
                int _e330 = level_14;
                keep_row(row_own, _e330);
                float2 cc_far = floor((p_10.xy / (float((int(1) << uint(far_level)))).xx));
                float2 cc_up = floor((p_10.xy / (float((int(1) << uint(up_level)))).xx));
                float2 cc_own = floor((p_10.xy / (fine).xx));
                Row _e351 = row_far;
                const uint _e356 = top_in(_e351, naga_f2i32(cc_far.x), naga_f2i32(cc_far.y));
                high_far = _e356;
                Row _e357 = row_up;
                const uint _e362 = top_in(_e357, naga_f2i32(cc_up.x), naga_f2i32(cc_up.y));
                high_up = _e362;
                Row _e363 = row_own;
                const uint _e368 = top_in(_e363, naga_f2i32(cc_own.x), naga_f2i32(cc_own.y));
                high_own = _e368;
            }
            k_7 = int(0);
            uint2 loop_bound_9 = uint2(4294967295u, 4294967295u);
            bool loop_init_7 = true;
            while(true) {
                if (all(loop_bound_9 == uint2(0u, 0u))) { break; }
                loop_bound_9 -= uint2(loop_bound_9.y == 0u, 1u);
                if (!loop_init_7) {
                    int _e452 = k_7;
                    k_7 = asint(asuint(_e452) + asuint(int(1)));
                }
                loop_init_7 = false;
                int _e371 = k_7;
                if ((_e371 < int(3))) {
                    bool _e376 = in_air;
                    local_55 = _e376;
                } else {
                    local_55 = false;
                }
                bool _e378 = local_55;
                if (_e378) {
                    bool _e381 = walking;
                    local_56 = !(_e381);
                } else {
                    local_56 = false;
                }
                bool _e384 = local_56;
                if (_e384) {
                } else {
                    break;
                }
                {
                    int _e385 = level_14;
                    int _e388 = k_7;
                    int up_3 = min(asint(asuint(asint(asuint(_e385) + asuint(int(4)))) - asuint(asint(asuint(int(2)) * asuint(_e388)))), TOP);
                    float up_size = float((int(1) << uint(up_3)));
                    float2 cc = floor((p_10.xy / (up_size).xx));
                    uint _e402 = high_own;
                    uint _e403 = high_up;
                    int _e404 = k_7;
                    uint _e408 = high_far;
                    int _e409 = k_7;
                    uint high_1 = ((_e409 == int(0)) ? _e408 : ((_e404 == int(1)) ? _e403 : _e402));
                    float top = ((floor(((float(high_1) - 1.0) / fine)) + 2.0) * fine);
                    if ((p_10.z <= top)) {
                        continue;
                    }
                    const float _e425 = leave2_(o_3.xy, d_9.xy, cc, up_size);
                    next = _e425;
                    if ((d_9.z < 0.0)) {
                        float _e430 = next;
                        float _e431 = t_2;
                        next = min(_e430, (_e431 + ((top - p_10.z) / d_9.z)));
                    }
                    float _e438 = next;
                    float _e439 = t_2;
                    if ((_e438 > (_e439 + 0.01))) {
                        float _e443 = next;
                        t_2 = (_e443 + 0.01);
                        float _e446 = t_2;
                        last_air = _e446;
                        walking = false;
                        chained = false;
                        float _e449 = t_2;
                        before_t = _e449;
                        skipped = true;
                        break;
                    }
                }
            }
            bool _e454 = skipped;
            if (_e454) {
                continue;
            }
            over = 0.0;
            int _e457 = level_14;
            if ((_e457 < TOP)) {
                float _e462 = t_2;
                float _e463 = lv_hand;
                if (!((_e462 > _e463))) {
                    int _e468 = level_14;
                    int _e469 = lv;
                    local_58 = (_e468 != _e469);
                } else {
                    local_58 = true;
                }
                bool _e472 = local_58;
                local_57 = _e472;
            } else {
                local_57 = false;
            }
            bool _e474 = local_57;
            if (_e474) {
                float _e475 = t_2;
                uint _e478 = layer_now;
                float _e482 = cameras[_e478].view.z;
                uint _e489 = layer_now;
                float _e493 = cameras[_e489].reach.y;
                int _e495 = level_14;
                over = ((log2(max((_e475 * _e482), 1.0)) + _e493) - float(_e495));
            }
            float _e498 = over;
            uint _e501 = layer_now;
            float _e505 = cameras[_e501].fade.z;
            uint _e509 = layer_now;
            float _e513 = cameras[_e509].fade.z;
            mixed = ((_e498 - _e505) / (1.0 - _e513));
            int _e518 = level_14;
            if ((_e518 < FIRST)) {
                float _e521 = mixed;
                int _e522 = level_14;
                const float _e523 = in_ring(_e522, p_10);
                int _e525 = level_14;
                mixed = max(_e521, (1.0 - (_e523 / (RING_FADE * float((int(1) << uint(_e525)))))));
            }
            look_8 = ZeroValueLook();
            float _e537 = mixed;
            if ((_e537 > 0.0)) {
                Place _e540 = place_6;
                int _e541 = level_14;
                const Spot _e542 = spot_kept(kept, _e540, _e541, p_10);
                Place _e543 = place_6;
                int _e544 = level_14;
                const Spot _e547 = spot_kept(kept_next, _e543, asint(asuint(_e544) + asuint(int(1))), p_10);
                if ((_e542.kind == int(2))) {
                    local_59 = (_e547.kind == int(2));
                } else {
                    local_59 = false;
                }
                bool _e557 = local_59;
                if (_e557) {
                    const Fetched _e559 = fetch_together(_e542.at);
                    const Fetched _e561 = fetch_together(_e547.at);
                    const Look _e562 = look_of(_e559);
                    look_8 = _e562;
                    Look _e563 = look_8;
                    const Look _e564 = look_of(_e561);
                    const Look _e568 = by_finer_cell(_e564, asint(asuint(_e547.level) - asuint(_e542.level)), d_9);
                    float _e569 = mixed;
                    const Look _e570 = mix_looks(_e563, _e568, _e569);
                    look_8 = _e570;
                } else {
                    const Look _e571 = look_at(_e542, p_10);
                    look_8 = _e571;
                    bool _e573 = look_8.known;
                    if (_e573) {
                        const Look _e574 = look_at(_e547, p_10);
                        if (_e574.known) {
                            Look _e576 = look_8;
                            const Look _e580 = by_finer_cell(_e574, asint(asuint(_e547.level) - asuint(_e542.level)), d_9);
                            float _e581 = mixed;
                            const Look _e582 = mix_looks(_e576, _e580, _e581);
                            look_8 = _e582;
                        }
                    }
                }
            } else {
                Place _e583 = place_6;
                int _e584 = level_14;
                const Look _e585 = far_blend_kept(kept, _e583, _e584, p_10);
                look_8 = _e585;
            }
            int _e586 = level_14;
            float size_5 = float((int(1) << uint(_e586)));
            float _e592 = look_8.solid;
            float _e594 = look_8.water;
            bool _e597 = look_8.known;
            float thick = (_e597 ? max(_e592, _e594) : 0.0);
            if ((thick > 0.0)) {
                bool _e604 = walking;
                local_60 = !(_e604);
            } else {
                local_60 = false;
            }
            bool _e607 = local_60;
            if (_e607) {
                float _e610 = t_2;
                float _e611 = last_air;
                local_61 = ((_e610 - _e611) > (size_5 * 0.26));
            } else {
                local_61 = false;
            }
            bool _e617 = local_61;
            if (_e617) {
                float _e618 = t_2;
                walk_to = _e618;
                float _e619 = last_air;
                t_2 = (_e619 + ((phase_1 * size_5) * 0.25));
                walking = true;
                in_air = false;
                chained = false;
                continue;
            }
            dt = (size_5 * 2.0);
            in_air = true;
            if ((thick > 0.0)) {
                in_air = false;
                walking = true;
                uint _e637 = layer_now;
                float _e641 = cameras[_e637].position.w;
                dt = ((thick > 0.02) ? (size_5 * surface_step) : (size_5 * _e641));
            } else {
                bool _e647 = walking;
                if (_e647) {
                    float _e650 = t_2;
                    float _e651 = walk_to;
                    local_62 = (_e650 < _e651);
                } else {
                    local_62 = false;
                }
                bool _e654 = local_62;
                if (_e654) {
                    in_air = false;
                    dt = (size_5 * surface_step);
                } else {
                    float _e657 = t_2;
                    last_air = _e657;
                    walking = false;
                    if ((d_9.z < 0.0)) {
                        float _e662 = dt;
                        dt = min(_e662, max((p_10.z / -(d_9.z)), (size_5 * 0.25)));
                    }
                }
            }
            bool _e672 = look_8.known;
            if (_e672) {
                bool _e675 = chained;
                local_63 = _e675;
            } else {
                local_63 = false;
            }
            bool _e677 = local_63;
            if (_e677) {
                bool _e681 = before.known;
                local_64 = _e681;
            } else {
                local_64 = false;
            }
            bool _e683 = local_64;
            if (_e683) {
                float _e684 = t_2;
                float _e686 = before_t;
                float length_ = (min(_e684, stop) - _e686);
                float _e689 = before.solid;
                float _e691 = look_8.solid;
                float solid_share = (0.5 * (_e689 + _e691));
                float _e696 = before.water;
                float _e698 = look_8.water;
                float water_share = (0.5 * (_e696 + _e698));
                solid_1 = 0.0;
                water = 0.0;
                covered = 0.0;
                gathers = ((solid_share + water_share) > 0.0);
                if (true) {
                    solid_1 = (1.0 - pow((1.0 - min(solid_share, 0.999)), (length_ / size_5)));
                    uint _e723 = layer_now;
                    float _e727 = cameras[_e723].water.w;
                    water = (1.0 - exp(((-(_e727) * water_share) * length_)));
                    float _e734 = solid_1;
                    float _e737 = water;
                    covered = (1.0 - ((1.0 - _e734) * (1.0 - _e737)));
                    float _e743 = covered;
                    gathers = (_e743 > 0.0);
                }
                bool _e746 = gathers;
                if (_e746) {
                    Look _e747 = before;
                    const float2 _e748 = faces_light(_e747, d_9);
                    Look _e749 = look_8;
                    const float2 _e750 = faces_light(_e749, d_9);
                    if (FACES) {
                        Look _e752 = before;
                        const float _e754 = shown_share(_e752, _e748.y);
                        Look _e755 = look_8;
                        const float _e757 = shown_share(_e755, _e750.y);
                        float shown_5 = (0.5 * (_e754 + _e757));
                        solid_1 = (1.0 - pow((1.0 - min(max(solid_share, shown_5), 0.999)), (length_ / size_5)));
                        uint _e772 = layer_now;
                        float _e776 = cameras[_e772].water.w;
                        water = (1.0 - exp(((-(_e776) * water_share) * length_)));
                        float _e783 = solid_1;
                        float _e786 = water;
                        covered = (1.0 - ((1.0 - _e783) * (1.0 - _e786)));
                    }
                    float _e792 = covered;
                    if ((_e792 > 0.0)) {
                        float _e795 = touched;
                        float _e796 = before_t;
                        touched = min(_e795, _e796);
                        float light_before = _e748.x;
                        float light_now = _e750.x;
                        float _e801 = before.solid;
                        float _e803 = look_8.solid;
                        float solids = max((_e801 + _e803), 1e-6);
                        float3 _e808 = before.colour;
                        const float3 _e809 = lit_(_e808, light_before);
                        float _e811 = before.solid;
                        float3 _e814 = look_8.colour;
                        const float3 _e815 = lit_(_e814, light_now);
                        float _e817 = look_8.solid;
                        float3 ground = (((_e809 * _e811) + (_e815 * _e817)) / (solids).xxx);
                        float3 _e823 = before.colour;
                        const float3 _e824 = sunlit(_e823, light_before);
                        float _e826 = before.solid;
                        float3 _e829 = look_8.colour;
                        const float3 _e830 = sunlit(_e829, light_now);
                        float _e832 = look_8.solid;
                        float3 ground_sun = (((_e824 * _e826) + (_e830 * _e832)) / (solids).xxx);
                        float _e837 = solid_1;
                        float _e838 = water;
                        float both = max((_e837 + _e838), 1e-6);
                        float _e842 = solid_1;
                        const float3 _e845 = lit_(water_colour, 1.0);
                        float _e846 = water;
                        float3 here_1 = (((ground * _e842) + (_e845 * _e846)) / (both).xxx);
                        float _e851 = solid_1;
                        const float3 _e854 = sunlit(water_colour, 1.0);
                        float _e855 = water;
                        float3 here_sun = (((ground_sun * _e851) + (_e854 * _e855)) / (both).xxx);
                        float3 _e860 = colour_6;
                        float _e861 = through;
                        float _e862 = covered;
                        colour_6 = (_e860 + ((_e861 * _e862) * here_1));
                        float3 _e866 = sunny;
                        float _e867 = through;
                        float _e868 = covered;
                        sunny = (_e866 + ((_e867 * _e868) * here_sun));
                        if ((p_10.z <= (size_5 * 1.5))) {
                            bottom = true;
                            bottom_here = here_1;
                            bottom_sun = here_sun;
                            float _e877 = t_2;
                            bottom_t = _e877;
                            int _e878 = level_14;
                            bottom_level = _e878;
                        }
                        float _e879 = through;
                        if ((_e879 > 0.5)) {
                            float _e882 = t_2;
                            first_1 = _e882;
                            int _e883 = level_14;
                            first_level_1 = _e883;
                        }
                        float _e884 = through;
                        float _e885 = covered;
                        through = (_e884 * (1.0 - _e885));
                    }
                }
            }
            Look _e889 = look_8;
            before = _e889;
            float _e890 = t_2;
            before_t = _e890;
            bool _e892 = look_8.known;
            chained = _e892;
            float _e893 = t_2;
            float _e894 = dt;
            t_2 = (_e893 + _e894);
        }
    }
    bool _e899 = floored;
    if (_e899) {
        bool _e902 = bottom;
        local_65 = _e902;
    } else {
        local_65 = false;
    }
    bool _e904 = local_65;
    if (_e904) {
        float3 _e905 = colour_6;
        float _e906 = through;
        float3 _e907 = bottom_here;
        colour_6 = (_e905 + (_e906 * _e907));
        float3 _e910 = sunny;
        float _e911 = through;
        float3 _e912 = bottom_sun;
        sunny = (_e910 + (_e911 * _e912));
        float _e915 = through;
        if ((_e915 > 0.5)) {
            float _e918 = bottom_t;
            first_1 = _e918;
            int _e919 = bottom_level;
            first_level_1 = _e919;
        }
        through = 0.0;
    }
    float reached = first_1;
    int reached_level = first_level_1;
    float _e923 = through;
    if ((_e923 > 0.5)) {
        uint _e928 = layer_now;
        float _e932 = cameras[_e928].reach.x;
        first_1 = _e932;
        first_level_1 = int(-1);
    }
    float3 gathered_1 = colour_6;
    float3 _e935 = colour_6;
    float _e936 = through;
    const float3 _e937 = sky(d_9);
    colour_6 = (_e935 + (_e936 * _e937));
    float _e940 = first_1;
    int _e941 = first_level_1;
    const float _e942 = shadow_wide(_e940, _e941);
    float3 _e943 = colour_6;
    float _e944 = first_1;
    int _e945 = steps_1;
    int _e947 = first_level_1;
    float3 _e948 = sunny;
    float _e949 = first_1;
    float3 _e952 = sunny;
    float _e954 = touched;
    float _e955 = through;
    bool _e956 = in_air;
    float _e957 = t_2;
    const Seen seen_5 = ConstructSeen(_e943, _e944, float(_e945), _e947, _e948, (o_3 + (d_9 * _e949)), _e942, (gathered_1 - _e952), _e954, _e955, reached, reached_level, _e956, _e957);
    return seen_5;
}

int naga_div(int lhs, int rhs) {
    return lhs / (((lhs == int(-2147483647 - 1) & rhs == -1) | (rhs == 0)) ? 1 : rhs);
}

void march_pixel(uint2 id_6, uint given_1, uint from_cut_1, uint to_cut_1, uint stack_1)
{
    bool local_66 = (bool)0;
    float shown_1 = 1.0;
    float begin_1 = 0.5;
    bool local_67 = (bool)0;
    float nearest = (float)0;
    int k_8 = int(0);
    float end_1 = (float)0;
    uint slot_6 = (uint)0;
    uint stops_at = (uint)0;
    uint put = NOT_CUT;
    float starts_1 = (float)0;
    float stop_1 = 1e30;
    float through0_1 = 1.0;
    bool look_first_1 = false;

    uint _e7 = layer_now;
    float4 _e10 = cameras[_e7].view;
    uint2 size_6 = naga_f2u32(_e10.xy);
    if (!((id_6.x >= size_6.x))) {
        local_66 = (id_6.y >= size_6.y);
    } else {
        local_66 = true;
    }
    bool _e23 = local_66;
    if (_e23) {
        return;
    }
    const uint2 _e26 = parent_of(id_6);
    uint _e29 = layer_now;
    float _e33 = cameras[_e29].layer.x;
    if ((_e33 > 0.0)) {
        uint _e39 = layer_now;
        float _e43 = cameras[_e39].layer.x;
        const uint _e48 = get_worth(((_e26.y * naga_f2u32(_e43)) + _e26.x));
        uint worth = (_e48 & 255u);
        shown_1 = (float(worth) / 255.0);
        if ((worth == 0u)) {
            if ((from_cut_1 == 0u)) {
                put_picture(int2(id_6.xy), (0.0).xxxx);
                uint _e66 = layer_now;
                float _e70 = cameras[_e66].reach.x;
                put_far(int2(id_6.xy), float4(_e70, 0.0, 0.0, 0.0));
                uint _e77 = layer_now;
                float _e81 = cameras[_e77].layer.y;
                if ((_e81 > 0.5)) {
                    put_worth(((id_6.y * size_6.x) + id_6.x), 0u);
                }
            }
            return;
        }
    }
    uint _e94 = layer_now;
    float _e98 = cameras[_e94].tiles.z;
    if ((_e98 > 0.0)) {
        uint _e105 = layer_now;
        float _e109 = cameras[_e105].tiles.z;
        uint2 tile_1 = naga_f2u32((float2(id_6.xy) / (_e109).xx));
        uint _e115 = layer_now;
        float _e119 = cameras[_e115].tiles.x;
        const float _e121 = tile_start(tile_1, naga_f2u32(_e119));
        begin_1 = _e121;
    }
    uint _e124 = layer_now;
    float _e128 = cameras[_e124].layer.x;
    if ((_e128 > 0.0)) {
        uint _e135 = layer_now;
        float _e139 = cameras[_e135].follow.x;
        local_67 = (_e139 > 0.5);
    } else {
        local_67 = false;
    }
    bool _e143 = local_67;
    if (_e143) {
        int2 above = int2(_e26);
        const int2 _e145 = far_above_edge();
        uint _e148 = layer_now;
        float _e152 = cameras[_e148].reach.x;
        nearest = _e152;
        uint2 loop_bound_10 = uint2(4294967295u, 4294967295u);
        bool loop_init_8 = true;
        while(true) {
            if (all(loop_bound_10 == uint2(0u, 0u))) { break; }
            loop_bound_10 -= uint2(loop_bound_10.y == 0u, 1u);
            if (!loop_init_8) {
                int _e178 = k_8;
                k_8 = asint(asuint(_e178) + asuint(int(1)));
            }
            loop_init_8 = false;
            int _e156 = k_8;
            if ((_e156 < int(9))) {
            } else {
                break;
            }
            {
                int _e159 = k_8;
                int _e164 = k_8;
                int2 at_13 = clamp(asint(asuint(above) + asuint(int2(asint(asuint(naga_mod(_e159, int(3))) - asuint(int(1))), asint(asuint(naga_div(_e164, int(3))) - asuint(int(1)))))), (int(0)).xx, _e145);
                float _e174 = nearest;
                const float _e175 = get_far_above(at_13);
                nearest = min(_e174, _e175);
            }
        }
        float _e180 = begin_1;
        float _e181 = nearest;
        float _e184 = nearest;
        uint _e187 = layer_now;
        float _e191 = cameras[_e187].view.z;
        begin_1 = max(_e180, ((_e181 - 2.0) - ((_e184 * _e191) * 4.0)));
    }
    float _e197 = begin_1;
    uint _e200 = layer_now;
    float _e204 = cameras[_e200].sky.w;
    begin_1 = max(_e197, _e204);
    const float3 _e206 = ray_through(id_6);
    uint _e209 = layer_now;
    float _e213 = cameras[_e209].reach.x;
    end_1 = _e213;
    uint _e217 = layer_now;
    float _e221 = cameras[_e217].follow.w;
    if ((_e221 > 0.5)) {
        uint _e226 = layer_now;
        float4 _e229 = cameras[_e226].position;
        const float _e231 = map_end(_e229.xyz, _e206);
        end_1 = _e231;
    }
    const float _e233 = ray_start(id_6.xy);
    slot_6 = given_1;
    stops_at = to_cut_1;
    if ((from_cut_1 > 0u)) {
        put = stack_1;
    } else {
        const uint _e240 = cut_slot(id_6);
        slot_6 = _e240;
        uint _e241 = slot_6;
        if ((_e241 != NOT_CUT)) {
            uint _e244 = slot_6;
            const uint _e245 = cut_first(_e244);
            stops_at = _e245;
            uint _e246 = slot_6;
            const uint _e247 = own_stack(_e246);
            put = _e247;
        }
    }
    float _e248 = begin_1;
    starts_1 = _e248;
    uint _e256 = put;
    if ((_e256 != NOT_CUT)) {
        const float _e265 = scatter(((id_6 * 2u) + uint2(1u, 1u)));
        const float _e266 = cut_place(from_cut_1, _e265);
        float _e267 = begin_1;
        starts_1 = max(_e267, _e266);
        uint _e269 = stops_at;
        if ((_e269 <= 16u)) {
            uint _e272 = stops_at;
            const float _e273 = cut_place(_e272, _e265);
            stop_1 = (_e273 + _e233);
        }
        float _e275 = begin_1;
        if ((_e266 > _e275)) {
            uint _e277 = slot_6;
            const float _e278 = cut_before(_e277, from_cut_1);
            through0_1 = clamp(abs(_e278), 0.01, 1.0);
            look_first_1 = (_e278 < 0.0);
        }
        uint _e285 = put;
        float _e286 = through0_1;
        float _e287 = starts_1;
        put_began(_e285, _e286, _e287);
    }
    uint _e290 = layer_now;
    float4 _e293 = cameras[_e290].position;
    float _e295 = starts_1;
    float _e296 = end_1;
    float _e297 = stop_1;
    float _e298 = through0_1;
    bool _e299 = look_first_1;
    const Seen _e300 = march(_e293.xyz, _e206, _e233, _e295, _e296, _e297, _e298, _e299);
    uint _e301 = put;
    if ((_e301 != NOT_CUT)) {
        uint _e304 = put;
        put_stack(_e304, _e300);
        return;
    }
    float _e305 = shown_1;
    finish(id_6, _e206, _e305, _e300, false, 0.0);
    return;
}

uint naga_div(uint lhs, uint rhs) {
    return lhs / (rhs == 0u ? 1u : rhs);
}

[numthreads(8, 8, 1)]
void march_all(uint3 id : SV_DispatchThreadID)
{
    uint2 at = (uint2)0;
    uint given = NOT_CUT;
    uint from_cut = 0u;
    uint to_cut = 17u;
    uint stack = 0u;
    bool local = (bool)0;
    uint rest = (uint)0;
    uint i = (uint)0;

    float _e5 = cameras[0].view.x;
    uint base_across = (naga_div((naga_f2u32(_e5) + 7u), 8u) * 8u);
    float _e17 = cameras[1].view.x;
    uint middle_across = (naga_div((naga_f2u32(_e17) + 7u), 8u) * 8u);
    float _e29 = cameras[2].view.x;
    uint fine_across = (naga_div((naga_f2u32(_e29) + 7u), 8u) * 8u);
    at = id.xy;
    if ((id.x < base_across)) {
        layer_now = 0u;
    } else {
        if ((id.x < (base_across + middle_across))) {
            uint _e55 = at.x;
            at.x = (_e55 - base_across);
            layer_now = 1u;
        } else {
            if ((id.x < ((base_across + middle_across) + fine_across))) {
                uint _e64 = at.x;
                at.x = (_e64 - (base_across + middle_across));
                layer_now = 2u;
            } else {
                float _e73 = cameras[0].cut.x;
                if ((_e73 <= 0.0)) {
                    return;
                }
                float _e81 = cameras[0].list.w;
                uint piece_1 = ((id.y * naga_f2u32(_e81)) + (id.x - ((base_across + middle_across) + fine_across)));
                float _e93 = cameras[0].cut.y;
                if ((piece_1 >= naga_f2u32(_e93))) {
                    return;
                }
                float _e101 = cameras[0].list.x;
                uint before_1 = (1u - naga_f2u32(_e101));
                const uint _e105 = pieces_at(before_1);
                uint slot_8 = asuint(cut.Load((_e105 + piece_1)*4));
                const uint3 _e109 = listed_pixel(slot_8);
                if ((_e109.z == 3u)) {
                    return;
                }
                const uint _e114 = list_at(before_1, slot_8);
                uint cuts_1 = asuint(cut.Load((_e114 + 1u)*4));
                const uint _e120 = list_at(before_1, slot_8);
                uint first_2 = asuint(cut.Load((_e120 + 2u)*4));
                if (!((piece_1 < first_2))) {
                    local = ((piece_1 - first_2) >= countbits(cuts_1));
                } else {
                    local = true;
                }
                bool _e133 = local;
                if (_e133) {
                    return;
                }
                rest = cuts_1;
                i = first_2;
                uint2 loop_bound_11 = uint2(4294967295u, 4294967295u);
                bool loop_init_9 = true;
                while(true) {
                    if (all(loop_bound_11 == uint2(0u, 0u))) { break; }
                    loop_bound_11 -= uint2(loop_bound_11.y == 0u, 1u);
                    if (!loop_init_9) {
                        uint _e144 = i;
                        i = (_e144 + 1u);
                    }
                    loop_init_9 = false;
                    uint _e136 = i;
                    if ((_e136 < piece_1)) {
                    } else {
                        break;
                    }
                    {
                        uint _e138 = rest;
                        uint _e139 = rest;
                        rest = (_e138 & (_e139 - 1u));
                    }
                }
                uint _e146 = rest;
                from_cut = (firstbitlow(_e146) + 1u);
                uint _e150 = rest;
                uint _e151 = rest;
                rest = (_e150 & (_e151 - 1u));
                uint _e155 = rest;
                uint _e160 = rest;
                to_cut = ((_e160 == 0u) ? 17u : (firstbitlow(_e155) + 1u));
                layer_now = _e109.z;
                at = _e109.xy;
                given = slot_8;
                const uint _e167 = piece_stack(piece_1);
                stack = _e167;
            }
        }
    }
    uint2 _e168 = at;
    uint _e169 = given;
    uint _e170 = from_cut;
    uint _e171 = to_cut;
    uint _e172 = stack;
    march_pixel(_e168, _e169, _e170, _e171, _e172);
    return;
}

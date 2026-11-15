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
RWByteAddressBuffer starts : register(u2);

int table(int level, int k)
{
    int row = ((level < FIRST) ? asint(asuint(int(12)) + asuint(asint(asuint(int(4)) - asuint(level)))) : asint(asuint(level) - asuint(FIRST)));
    int _e16 = asint(levels.Load(asint(asuint(asint(asuint(row) * asuint(TABLE))) + asuint(k))*4));
    return _e16;
}

float world_top()
{
    float _e3 = camera.reach.w;
    return min(max(_e3, 0.0), WORLD_HEIGHT);
}

uint entry_at(int level_1, int cx, int cy)
{
    bool local_1 = (bool)0;
    bool local_2 = (bool)0;
    bool local_3 = (bool)0;

    const int _e4 = table(level_1, int(1));
    int x = asint(asuint(cx) - asuint(_e4));
    const int _e7 = table(level_1, int(2));
    int y = asint(asuint(cy) - asuint(_e7));
    const int _e10 = table(level_1, int(3));
    if (!((x < int(0)))) {
        local_1 = (y < int(0));
    } else {
        local_1 = true;
    }
    bool _e19 = local_1;
    if (!(_e19)) {
        local_2 = (x >= _e10);
    } else {
        local_2 = true;
    }
    bool _e25 = local_2;
    if (!(_e25)) {
        local_3 = (y >= _e10);
    } else {
        local_3 = true;
    }
    bool _e31 = local_3;
    if (_e31) {
        return PAST_MAP;
    }
    const int _e35 = table(level_1, int(4));
    uint _e41 = asuint(entries.Load(uint(asint(asuint(asint(asuint(_e35) + asuint(asint(asuint(y) * asuint(_e10))))) + asuint(x)))*4));
    return _e41;
}

float leave2_(float2 o, float2 d, float2 c, float size)
{
    float2 next_1 = ((c + ((d > (0.0).xx) ? (1.0).xx : (0.0).xx)) * size);
    float2 t_1 = ((next_1 - o) / d);
    float tx = ((abs(d.x) > 1e-6) ? t_1.x : 1e30);
    float ty = ((abs(d.y) > 1e-6) ? t_1.y : 1e30);
    return min(tx, ty);
}

int naga_f2i32(float value) {
    return int(clamp(value, -2147483600.0, 2147483500.0));
}

float cone_start(float3 o_1, float3 d_1, float spread)
{
    float t = 0.5;
    int n = int(0);
    bool local_4 = (bool)0;
    bool advanced = (bool)0;
    int k_1 = (int)0;
    bool local_5 = (bool)0;
    float top = (float)0;
    float next = (float)0;

    float across = length(d_1.xy);
    const float _e8 = world_top();
    float ceiling = (_e8 + 32.0);
    uint2 loop_bound = uint2(4294967295u, 4294967295u);
    bool loop_init = true;
    while(true) {
        if (all(loop_bound == uint2(0u, 0u))) { break; }
        loop_bound -= uint2(loop_bound.y == 0u, 1u);
        if (!loop_init) {
            int _e180 = n;
            n = asint(asuint(_e180) + asuint(int(1)));
        }
        loop_init = false;
        int _e12 = n;
        if ((_e12 < int(96))) {
        } else {
            break;
        }
        {
            float _e15 = t;
            float _e19 = camera.reach.x;
            if ((_e15 >= _e19)) {
                float _e24 = camera.reach.x;
                return _e24;
            }
            float _e25 = t;
            float3 p = (o_1 + (d_1 * _e25));
            float _e29 = t;
            if (((p.z - (_e29 * spread)) > ceiling)) {
                local_4 = (d_1.z >= 0.0);
            } else {
                local_4 = false;
            }
            bool _e39 = local_4;
            if (_e39) {
                float _e43 = camera.reach.x;
                return _e43;
            }
            float _e44 = t;
            float r = ((_e44 * spread) + 2.0);
            float2 rg = floor((p.xy / (1024.0).xx));
            const uint _e58 = entry_at(REGION, naga_f2i32(rg.x), naga_f2i32(rg.y));
            if ((_e58 == NOT_SUMMED)) {
                float _e61 = t;
                return _e61;
            }
            advanced = false;
            int first_level = clamp(naga_f2i32(ceil(log2((4.0 * r)))), FIRST, TOP);
            k_1 = int(0);
            uint2 loop_bound_1 = uint2(4294967295u, 4294967295u);
            bool loop_init_1 = true;
            while(true) {
                if (all(loop_bound_1 == uint2(0u, 0u))) { break; }
                loop_bound_1 -= uint2(loop_bound_1.y == 0u, 1u);
                if (!loop_init_1) {
                    int _e174 = k_1;
                    k_1 = asint(asuint(_e174) + asuint(int(1)));
                }
                loop_init_1 = false;
                int _e74 = k_1;
                if ((_e74 < int(3))) {
                } else {
                    break;
                }
                {
                    int _e77 = k_1;
                    int level_2 = min(asint(asuint(first_level) + asuint(_e77)), TOP);
                    float size_1 = float((int(1) << uint(level_2)));
                    float2 c_1 = floor((p.xy / (size_1).xx));
                    if (!(any(((p.xy - (r).xx) < (c_1 * size_1))))) {
                        local_5 = any(((p.xy + (r).xx) >= ((c_1 + (1.0).xx) * size_1)));
                    } else {
                        local_5 = true;
                    }
                    bool _e108 = local_5;
                    if (_e108) {
                        continue;
                    }
                    const uint _e113 = entry_at(level_2, naga_f2i32(c_1.x), naga_f2i32(c_1.y));
                    if ((_e113 == NOT_SUMMED)) {
                        continue;
                    }
                    top = -1000000000.0;
                    if ((_e113 != PAST_MAP)) {
                        int low = (asint((_e113 << 16u)) >> 16u);
                        int count = int((_e113 >> 16u));
                        top = (float(asint(asuint(low) + asuint(count))) * size_1);
                    }
                    float _e133 = top;
                    if (((p.z - r) <= (_e133 + 32.0))) {
                        break;
                    }
                    const float _e138 = leave2_(o_1.xy, d_1.xy, c_1, size_1);
                    next = (((_e138 * across) - 2.0) / max((across + spread), 1e-6));
                    if (((d_1.z - spread) < 0.0)) {
                        float _e151 = next;
                        float _e152 = top;
                        float _e159 = t;
                        next = min(_e151, (((((_e152 + 32.0) + 2.0) - p.z) + (d_1.z * _e159)) / (d_1.z - spread)));
                    }
                    float _e166 = next;
                    float _e167 = t;
                    if ((_e166 > (_e167 + 1.0))) {
                        float _e171 = next;
                        t = _e171;
                        advanced = true;
                        break;
                    }
                }
            }
            bool _e176 = advanced;
            if (!(_e176)) {
                float _e178 = t;
                return _e178;
            }
        }
    }
    float _e182 = t;
    return _e182;
}

uint2 naga_f2u32(float2 value) {
    return uint2(clamp(value, 0.0, 4294967000.0));
}

[numthreads(8, 8, 1)]
void start_pass(uint3 id : SV_DispatchThreadID)
{
    bool local = (bool)0;

    float4 _e3 = camera.tiles;
    uint2 tiles = naga_f2u32(_e3.xy);
    if (!((id.x >= tiles.x))) {
        local = (id.y >= tiles.y);
    } else {
        local = true;
    }
    bool _e16 = local;
    if (_e16) {
        return;
    }
    float _e25 = camera.tiles.z;
    float4 _e29 = camera.view;
    float2 uv = min((((float2(id.xy) + (0.5).xx) * _e25) / _e29.xy), (1.0).xx);
    float4 _e37 = camera.forward;
    float4 _e46 = camera.right;
    float4 _e57 = camera.up;
    float3 d_2 = normalize(((_e37.xyz + (((uv.x * 2.0) - 1.0) * _e46.xyz)) + ((1.0 - (uv.y * 2.0)) * _e57.xyz)));
    float4 _e71 = camera.position;
    float _e76 = camera.tiles.w;
    const float _e77 = cone_start(_e71.xyz, d_2, _e76);
    starts.Store(((id.y * tiles.x) + id.x)*4, asuint(_e77));
    return;
}

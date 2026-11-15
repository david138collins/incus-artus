struct NagaConstants {
    int first_vertex;
    int first_instance;
    uint other;
};
ConstantBuffer<NagaConstants> _NagaConstants: register(b1);

struct Up {
    float4 size;
    float4 mode;
};

Texture2D<float4> layer8_ : register(t0);
Texture2D<float4> layer4_ : register(t1);
Texture2D<float4> layer2_ : register(t2);
Texture2D<float4> far8_ : register(t3);
SamplerState nagaSamplerHeap[2048]: register(s0, space0);
SamplerComparisonState nagaComparisonSamplerHeap[2048]: register(s2048, space0);
StructuredBuffer<uint> nagaGroup0SamplerIndexArray : register(t4, space0);
static const SamplerState blended = nagaSamplerHeap[nagaGroup0SamplerIndexArray[0]];
cbuffer up : register(b0) { Up up; }

struct FragmentInput_brought_up {
    float4 at_2 : SV_Position;
};

int2 naga_f2i32(float2 value) {
    return int2(clamp(value, -2147483600.0, 2147483500.0));
}

uint2 NagaDimensions2D(Texture2D<float4> tex)
{
    uint4 ret;
    tex.GetDimensions(0, ret.x, ret.y, ret.z);
    return ret.xy;
}

float4 read(Texture2D<float4> layer, float factor, float2 at_1, bool smooth_it_1)
{
    float4 c = (float4)0;

    int2 size = int2(NagaDimensions2D(layer));
    float2 q = (at_1 / (factor).xx);
    float4 _e17 = layer.Load(int3(clamp(naga_f2i32(floor(q)), (int(0)).xx, asint(asuint(size) - asuint((int(1)).xx))), int(0)));
    c = _e17;
    if (smooth_it_1) {
        float4 _e23 = layer.SampleLevel(blended, (q / float2(size)), 0.0);
        c = _e23;
    }
    float _e25 = c.w;
    if ((_e25 < 0.001)) {
        return (0.0).xxxx;
    }
    float4 _e30 = c;
    float _e33 = c.w;
    float _e37 = c.w;
    return float4((_e30.xyz / (_e33).xxx), _e37);
}

uint naga_f2u32(float value) {
    return uint(clamp(value, 0.0, 4294967000.0));
}

float4 brought_up(FragmentInput_brought_up fragmentinput_brought_up) : SV_Target0
{
    float4 at = fragmentinput_brought_up.at_2;
    bool smooth_it = (bool)0;
    float near = 1e30;
    float far = 0.0;
    int k = int(0);
    float3 colour = (float3)0;

    float _e4 = up.mode.x;
    uint mode = naga_f2u32(_e4);
    smooth_it = (mode != 0u);
    float finest = up.size.z;
    float coarsest = up.size.w;
    if ((mode == 2u)) {
        float4 _e21 = up.mode;
        int2 size_1 = naga_f2i32(_e21.yz);
        int2 b = naga_f2i32(floor(((at.xy / (coarsest).xx) - (0.5).xx)));
        uint2 loop_bound = uint2(4294967295u, 4294967295u);
        bool loop_init = true;
        while(true) {
            if (all(loop_bound == uint2(0u, 0u))) { break; }
            loop_bound -= uint2(loop_bound.y == 0u, 1u);
            if (!loop_init) {
                int _e64 = k;
                k = asint(asuint(_e64) + asuint(int(1)));
            }
            loop_init = false;
            int _e38 = k;
            if ((_e38 < int(4))) {
            } else {
                break;
            }
            {
                int _e42 = k;
                int _e45 = k;
                float4 _e57 = far8_.Load(int3(clamp(asint(asuint(b) + asuint(int2((_e42 & int(1)), (_e45 >> 1u)))), (int(0)).xx, asint(asuint(size_1) - asuint((int(1)).xx))), int(0)));
                float d = _e57.x;
                float _e59 = near;
                near = min(_e59, d);
                float _e61 = far;
                far = max(_e61, d);
            }
        }
        float _e66 = far;
        float _e67 = near;
        smooth_it = !((_e66 > ((_e67 * 1.5) + 8.0)));
    }
    bool _e76 = smooth_it;
    const float4 _e77 = read(layer8_, coarsest, at.xy, _e76);
    colour = _e77.xyz;
    if ((finest <= (coarsest * 0.5))) {
        bool _e87 = smooth_it;
        const float4 _e88 = read(layer4_, (coarsest * 0.5), at.xy, _e87);
        float3 _e89 = colour;
        colour = lerp(_e89, _e88.xyz, _e88.w);
    }
    if ((finest <= (coarsest * 0.25))) {
        bool _e100 = smooth_it;
        const float4 _e101 = read(layer2_, (coarsest * 0.25), at.xy, _e100);
        float3 _e102 = colour;
        colour = lerp(_e102, _e101.xyz, _e101.w);
    }
    float3 _e106 = colour;
    return float4(_e106, 1.0);
}

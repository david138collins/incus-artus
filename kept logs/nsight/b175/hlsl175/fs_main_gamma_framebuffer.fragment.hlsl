struct NagaConstants {
    int first_vertex;
    int first_instance;
    uint other;
};
ConstantBuffer<NagaConstants> _NagaConstants: register(b1);

struct VertexOutput {
    float2 tex_coord : LOC0;
    float4 color : LOC1;
    float4 position : SV_Position;
};

struct Locals {
    float2 screen_size;
    uint dithering;
    uint predictable_texture_filtering;
};

cbuffer r_locals : register(b0) { Locals r_locals; }
Texture2D<float4> r_tex_color : register(t0);
SamplerState nagaSamplerHeap[2048]: register(s0, space0);
SamplerComparisonState nagaComparisonSamplerHeap[2048]: register(s2048, space0);
StructuredBuffer<uint> nagaGroup1SamplerIndexArray : register(t1, space0);
static const SamplerState r_tex_sampler = nagaSamplerHeap[nagaGroup1SamplerIndexArray[0]];

struct FragmentInput_fs_main_gamma_framebuffer {
    float2 tex_coord : LOC0;
    float4 color : LOC1;
    float4 position : SV_Position;
};

float interleaved_gradient_noise(float2 n)
{
    float f = ((0.06711056 * n.x) + (0.00583715 * n.y));
    return frac((52.982918 * frac(f)));
}

float3 dither_interleaved(float3 rgb, float levels, float4 frag_coord)
{
    float noise_ = (float)0;

    const float _e4 = interleaved_gradient_noise(frag_coord.xy);
    noise_ = _e4;
    float _e6 = noise_;
    noise_ = ((_e6 - 0.5) * 0.95);
    float _e11 = noise_;
    return (rgb + ((_e11 / (levels - 1.0))).xxx);
}

int2 naga_f2i32(float2 value) {
    return int2(clamp(value, -2147483600.0, 2147483500.0));
}

uint2 NagaMipDimensions2D(Texture2D<float4> tex, uint mip_level)
{
    uint4 ret;
    tex.GetDimensions(mip_level, ret.x, ret.y, ret.z);
    return ret.xy;
}

float4 sample_texture(VertexOutput in_1)
{
    uint _e3 = r_locals.predictable_texture_filtering;
    if ((_e3 == 0u)) {
        float4 _e9 = r_tex_color.Sample(r_tex_sampler, in_1.tex_coord);
        return _e9;
    } else {
        int2 texture_size = int2(NagaMipDimensions2D(r_tex_color, int(0)));
        float2 texture_size_f = float2(texture_size);
        float2 pixel_coord = ((in_1.tex_coord * texture_size_f) - (0.5).xx);
        float2 pixel_fract = frac(pixel_coord);
        int2 pixel_floor = naga_f2i32(floor(pixel_coord));
        int2 max_coord = asint(asuint(texture_size) - asuint(int2(int(1), int(1))));
        int2 p00_ = clamp(asint(asuint(pixel_floor) + asuint(int2(int(0), int(0)))), int2(int(0), int(0)), max_coord);
        int2 p10_ = clamp(asint(asuint(pixel_floor) + asuint(int2(int(1), int(0)))), int2(int(0), int(0)), max_coord);
        int2 p01_ = clamp(asint(asuint(pixel_floor) + asuint(int2(int(0), int(1)))), int2(int(0), int(0)), max_coord);
        int2 p11_ = clamp(asint(asuint(pixel_floor) + asuint(int2(int(1), int(1)))), int2(int(0), int(0)), max_coord);
        float4 tl = r_tex_color.Load(int3(p00_, int(0)));
        float4 tr = r_tex_color.Load(int3(p10_, int(0)));
        float4 bl = r_tex_color.Load(int3(p01_, int(0)));
        float4 br = r_tex_color.Load(int3(p11_, int(0)));
        float4 top = lerp(tl, tr, pixel_fract.x);
        float4 bottom = lerp(bl, br, pixel_fract.x);
        return lerp(top, bottom, pixel_fract.y);
    }
}

float4 fs_main_gamma_framebuffer(FragmentInput_fs_main_gamma_framebuffer fragmentinput_fs_main_gamma_framebuffer) : SV_Target0
{
    VertexOutput in_ = { fragmentinput_fs_main_gamma_framebuffer.tex_coord, fragmentinput_fs_main_gamma_framebuffer.color, fragmentinput_fs_main_gamma_framebuffer.position };
    float4 out_color_gamma = (float4)0;

    const float4 _e1 = sample_texture(in_);
    out_color_gamma = (in_.color * _e1);
    uint _e7 = r_locals.dithering;
    if ((_e7 == 1u)) {
        float4 _e10 = out_color_gamma;
        const float3 _e14 = dither_interleaved(_e10.xyz, 256.0, in_.position);
        float _e16 = out_color_gamma.w;
        out_color_gamma = float4(_e14, _e16);
    }
    float4 _e18 = out_color_gamma;
    return _e18;
}

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

struct VertexOutput_vs_main {
    float2 tex_coord : LOC0;
    float4 color_1 : LOC1;
    float4 position : SV_Position;
};

float4 unpack_color(uint color)
{
    return (float4(float((color & 255u)), float(((color >> 8u) & 255u)), float(((color >> 16u) & 255u)), float(((color >> 24u) & 255u))) / (255.0).xxxx);
}

float4 position_from_screen(float2 screen_pos)
{
    float _e7 = r_locals.screen_size.x;
    float _e17 = r_locals.screen_size.y;
    return float4((((2.0 * screen_pos.x) / _e7) - 1.0), (1.0 - ((2.0 * screen_pos.y) / _e17)), 0.0, 1.0);
}

VertexOutput_vs_main vs_main(float2 a_pos : LOC0, float2 a_tex_coord : LOC1, uint a_color : LOC2)
{
    VertexOutput out_ = (VertexOutput)0;

    out_.tex_coord = a_tex_coord;
    const float4 _e6 = unpack_color(a_color);
    out_.color = _e6;
    const float4 _e8 = position_from_screen(a_pos);
    out_.position = _e8;
    VertexOutput _e9 = out_;
    const VertexOutput vertexoutput = _e9;
    const VertexOutput_vs_main vertexoutput_1 = { vertexoutput.tex_coord, vertexoutput.color, vertexoutput.position };
    return vertexoutput_1;
}

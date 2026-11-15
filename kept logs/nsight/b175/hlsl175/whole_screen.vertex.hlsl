struct NagaConstants {
    int first_vertex;
    int first_instance;
    uint other;
};
ConstantBuffer<NagaConstants> _NagaConstants: register(b1);

float4 whole_screen(uint k : SV_VertexID) : SV_Position
{
    float2 uv = float2(float((((_NagaConstants.first_vertex + k) << 1u) & 2u)), float(((_NagaConstants.first_vertex + k) & 2u)));
    return float4(((uv * 2.0) - (1.0).xx), 0.0, 1.0);
}

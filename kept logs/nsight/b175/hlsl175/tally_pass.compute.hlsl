struct NagaConstants {
    int first_vertex;
    int first_instance;
    uint other;
};
ConstantBuffer<NagaConstants> _NagaConstants: register(b1);

Texture2D<float4> steps : register(t0);
RWByteAddressBuffer counts : register(u0);
cbuffer part : register(b0) { uint4 part; }

uint naga_f2u32(float value) {
    return uint(clamp(value, 0.0, 4294967000.0));
}

[numthreads(8, 8, 1)]
void tally_pass(uint3 id : SV_DispatchThreadID)
{
    bool local = (bool)0;

    uint _e4 = part.z;
    if (!((id.x >= _e4))) {
        uint _e12 = part.w;
        local = (id.y >= _e12);
    } else {
        local = true;
    }
    bool _e15 = local;
    if (_e15) {
        return;
    }
    uint4 _e19 = part;
    float4 _e24 = steps.Load(int3(int2((id.xy + _e19.xy)), int(0)));
    uint n = naga_f2u32(_e24.y);
    uint _e29; counts.InterlockedMax(0, n, _e29);
    if ((n > 256u)) {
        uint _e35; counts.InterlockedAdd(4, 1u, _e35);
    }
    if ((n > 384u)) {
        uint _e41; counts.InterlockedAdd(8, 1u, _e41);
    }
    if ((n > 512u)) {
        uint _e47; counts.InterlockedAdd(12, 1u, _e47);
        return;
    } else {
        return;
    }
}

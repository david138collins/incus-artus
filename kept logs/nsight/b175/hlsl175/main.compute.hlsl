struct __dynamic_buffer_offsetsTy1 {
    uint _0;
};
ConstantBuffer<__dynamic_buffer_offsetsTy1> __dynamic_buffer_offsets1: register(b1, space1);

struct OffsetPc {
    uint inner;
};

RWByteAddressBuffer dst_ : register(u0);
ByteAddressBuffer src : register(t0);
ConstantBuffer<OffsetPc> offset: register(b0);

typedef uint ret_Constructarray6_uint_[6];
ret_Constructarray6_uint_ Constructarray6_uint_(uint arg0, uint arg1, uint arg2, uint arg3, uint arg4, uint arg5) {
    uint ret[6] = { arg0, arg1, arg2, arg3, arg4, arg5 };
    return ret;
}

[numthreads(1, 1, 1)]
void main()
{
    bool local = (bool)0;
    bool local_1 = (bool)0;

    uint _e3 = offset.inner;
    uint _e5 = asuint(src.Load(_e3*4+__dynamic_buffer_offsets1._0));
    uint _e9 = offset.inner;
    uint _e13 = asuint(src.Load((_e9 + 1u)*4+__dynamic_buffer_offsets1._0));
    uint _e17 = offset.inner;
    uint _e21 = asuint(src.Load((_e17 + 2u)*4+__dynamic_buffer_offsets1._0));
    uint3 src_1 = uint3(_e5, _e13, _e21);
    if (!((src_1.x > 65535u))) {
        local = (src_1.y > 65535u);
    } else {
        local = true;
    }
    bool _e32 = local;
    if (!(_e32)) {
        local_1 = (src_1.z > 65535u);
    } else {
        local_1 = true;
    }
    bool _e39 = local_1;
    if (_e39) {
        {
            uint _value3[6] = Constructarray6_uint_(0u, 0u, 0u, 0u, 0u, 0u);
            dst_.Store(0, asuint(_value3[0]));
            dst_.Store(4, asuint(_value3[1]));
            dst_.Store(8, asuint(_value3[2]));
            dst_.Store(12, asuint(_value3[3]));
            dst_.Store(16, asuint(_value3[4]));
            dst_.Store(20, asuint(_value3[5]));
        }
        return;
    } else {
        {
            uint _value3[6] = Constructarray6_uint_(src_1.x, src_1.y, src_1.z, src_1.x, src_1.y, src_1.z);
            dst_.Store(0, asuint(_value3[0]));
            dst_.Store(4, asuint(_value3[1]));
            dst_.Store(8, asuint(_value3[2]));
            dst_.Store(12, asuint(_value3[3]));
            dst_.Store(16, asuint(_value3[4]));
            dst_.Store(20, asuint(_value3[5]));
        }
        return;
    }
}

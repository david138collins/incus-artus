struct __dynamic_buffer_offsetsTy1 {
    uint _0;
};
ConstantBuffer<__dynamic_buffer_offsetsTy1> __dynamic_buffer_offsets1: register(b1, space1);

struct MetadataEntry {
    uint src_offset;
    uint dst_offset;
    uint vertex_or_index_limit;
    uint instance_limit;
};

struct MetadataRange {
    uint start;
    uint count;
};

static const uint OFFSET_MASK = 1073741823u;
static const bool supports_indirect_first_instance = false;
static const bool write_d3d12_special_constants = true;

ConstantBuffer<MetadataRange> metadata_range: register(b0);
ByteAddressBuffer metadata : register(t0);
ByteAddressBuffer src : register(t1);
RWByteAddressBuffer dst_ : register(u0);

bool is_bit_set(uint data, uint index)
{
    return (((data >> index) & 1u) == 1u);
}

MetadataEntry ConstructMetadataEntry(uint arg0, uint arg1, uint arg2, uint arg3) {
    MetadataEntry ret = (MetadataEntry)0;
    ret.src_offset = arg0;
    ret.dst_offset = arg1;
    ret.vertex_or_index_limit = arg2;
    ret.instance_limit = arg3;
    return ret;
}

[numthreads(64, 1, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID)
{
    bool failed = false;
    bool local = (bool)0;
    bool local_1 = (bool)0;

    uint _e4 = metadata_range.count;
    if ((global_invocation_id.x >= _e4)) {
        return;
    }
    uint _e9 = metadata_range.start;
    MetadataEntry metadata_1 = ConstructMetadataEntry(asuint(metadata.Load((_e9 + global_invocation_id.x)*16+0)), asuint(metadata.Load((_e9 + global_invocation_id.x)*16+4)), asuint(metadata.Load((_e9 + global_invocation_id.x)*16+8)), asuint(metadata.Load((_e9 + global_invocation_id.x)*16+12)));
    const bool _e18 = is_bit_set(metadata_1.src_offset, 31u);
    uint src_base_offset = (metadata_1.src_offset & OFFSET_MASK);
    uint dst_base_offset = (metadata_1.dst_offset & OFFSET_MASK);
    uint first_vertex_or_index = asuint(src.Load((src_base_offset + 2u)*4+__dynamic_buffer_offsets1._0));
    uint vertex_or_index_count = asuint(src.Load((src_base_offset + 0u)*4+__dynamic_buffer_offsets1._0));
    {
        const bool _e37 = is_bit_set(metadata_1.dst_offset, 30u);
        bool sub_overflows = (metadata_1.vertex_or_index_limit < first_vertex_or_index);
        bool _e40 = failed;
        if (sub_overflows) {
            local = !(_e37);
        } else {
            local = false;
        }
        bool _e45 = local;
        failed = (_e40 | _e45);
        uint vertex_or_index_limit = (metadata_1.vertex_or_index_limit - first_vertex_or_index);
        bool _e49 = failed;
        failed = (_e49 | (vertex_or_index_limit < vertex_or_index_count));
    }
    uint first_instance = asuint(src.Load(((src_base_offset + 3u) + uint(_e18))*4+__dynamic_buffer_offsets1._0));
    uint instance_count = asuint(src.Load((src_base_offset + 1u)*4+__dynamic_buffer_offsets1._0));
    {
        const bool _e66 = is_bit_set(metadata_1.dst_offset, 31u);
        bool sub_overflows_1 = (metadata_1.instance_limit < first_instance);
        bool _e69 = failed;
        if (sub_overflows_1) {
            local_1 = !(_e66);
        } else {
            local_1 = false;
        }
        bool _e74 = local_1;
        failed = (_e69 | _e74);
        uint instance_limit = (metadata_1.instance_limit - first_instance);
        bool _e78 = failed;
        failed = (_e78 | (instance_limit < instance_count));
    }
    if (true) {
        bool _e84 = failed;
        failed = (_e84 | (first_instance != 0u));
    }
    bool _e93 = failed;
    if (_e93) {
        if (write_d3d12_special_constants) {
            dst_.Store((dst_base_offset + 0u)*4, asuint(0u));
            dst_.Store((dst_base_offset + 1u)*4, asuint(0u));
            dst_.Store((dst_base_offset + 2u)*4, asuint(0u));
        }
        dst_.Store(((dst_base_offset + 3u) + 0u)*4, asuint(0u));
        dst_.Store(((dst_base_offset + 3u) + 1u)*4, asuint(0u));
        dst_.Store(((dst_base_offset + 3u) + 2u)*4, asuint(0u));
        dst_.Store(((dst_base_offset + 3u) + 3u)*4, asuint(0u));
        if (_e18) {
            dst_.Store(((dst_base_offset + 3u) + 4u)*4, asuint(0u));
            return;
        } else {
            return;
        }
    } else {
        if (write_d3d12_special_constants) {
            uint _e151 = asuint(src.Load(((src_base_offset + 2u) + uint(_e18))*4+__dynamic_buffer_offsets1._0));
            dst_.Store((dst_base_offset + 0u)*4, asuint(_e151));
            uint _e162 = asuint(src.Load(((src_base_offset + 3u) + uint(_e18))*4+__dynamic_buffer_offsets1._0));
            dst_.Store((dst_base_offset + 1u)*4, asuint(_e162));
            dst_.Store((dst_base_offset + 2u)*4, asuint(0u));
        }
        uint _e177 = asuint(src.Load((src_base_offset + 0u)*4+__dynamic_buffer_offsets1._0));
        dst_.Store(((dst_base_offset + 3u) + 0u)*4, asuint(_e177));
        uint _e187 = asuint(src.Load((src_base_offset + 1u)*4+__dynamic_buffer_offsets1._0));
        dst_.Store(((dst_base_offset + 3u) + 1u)*4, asuint(_e187));
        uint _e197 = asuint(src.Load((src_base_offset + 2u)*4+__dynamic_buffer_offsets1._0));
        dst_.Store(((dst_base_offset + 3u) + 2u)*4, asuint(_e197));
        uint _e207 = asuint(src.Load((src_base_offset + 3u)*4+__dynamic_buffer_offsets1._0));
        dst_.Store(((dst_base_offset + 3u) + 3u)*4, asuint(_e207));
        if (_e18) {
            uint _e217 = asuint(src.Load((src_base_offset + 4u)*4+__dynamic_buffer_offsets1._0));
            dst_.Store(((dst_base_offset + 3u) + 4u)*4, asuint(_e217));
            return;
        } else {
            return;
        }
    }
}

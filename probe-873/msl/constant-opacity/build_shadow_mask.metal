// language: metal4.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size4;
    uint size5;
    uint size11;
    uint size13;
};

struct type_4 {
    metal::float2 inner[4];
};
struct CameraUniform {
    metal::float4x4 view_proj;
    metal::packed_int3 anchor;
    float exposure;
    metal::float3 offset;
    metal::packed_int3 tlas_anchor;
    uint dynamic_tlas_entries;
    metal::float3 dynamic_tlas_bounds_min;
    metal::float3 dynamic_tlas_bounds_max;
};
struct LightUniform {
    metal::float4 ambient_illuminance;
    uint directional_light_count;
    uint point_light_count;
    char _pad3[8];
};
struct LightData {
    metal::packed_int3 anchor;
    float range;
    metal::float3 offset;
    metal::float3 intensity;
};
typedef LightData type_10[1];
struct type_11 {
    uint inner[127];
};
struct FroxelList {
    uint light_count;
    type_11 lights;
};
typedef FroxelList type_12[1];
typedef uint type_18[1];
struct PixelList {
    metal::atomic_uint count;
    type_18 entries;
};
struct type_19 {
    metal::atomic_uint inner[3];
};
struct RayIntersection {
    uint kind;
    float t;
    uint instance_custom_data;
    uint instance_index;
    uint sbt_record_offset;
    uint geometry_index;
    uint primitive_index;
    metal::float2 barycentrics;
    bool front_face;
    char _pad9[11];
    metal::float4x3 object_to_world;
    metal::float4x3 world_to_object;
};
struct RayDesc {
    uint flags;
    uint cull_mask;
    float tmin;
    float tmax;
    metal::float3 origin;
    metal::float3 dir;
};
struct SampleSurface {
    metal::packed_float3 position;
    float pixel_width;
    metal::float3 triangle_normal;
    metal::float3 interpolated_normal;
};
struct SurfaceWord {
    metal::uint2 word;
    uint traced_directional_lights;
    char _pad2[4];
};
constant float ANCHOR_CELL_SIZE = 64.0;
constant uint MESHLET_VERTEX_BITS = 6u;
constant uint MESHLET_MAX_TRIANGLES = 124u;
constant uint MESH_MAX_LODS = 8u;
constant uint NO_MESH = 4294967295u;
constant uint NO_MATERIAL = 4294967295u;
constant uint SCAN_BLOCK_SIZE = 256u;
constant uint MAX_WORKGROUPS_PER_DIMENSION = 65535u;
constant uint INSTANCE_CAPACITY = 517376u;
constant uint DEPTH_PYRAMID_TILE_SIZE = 8u;
constant uint LIGHT_COUNT_TILE_SIZE = 8u;
constant metal::float3 LUMINANCE_COEFFICIENTS = metal::float3(0.2126, 0.7152, 0.0722);
constant uint DITHER_MASK_SIDE = 64u;
constant uint FROXEL_COLUMNS = 32u;
constant uint FROXEL_ROWS = 18u;
constant uint FROXEL_SLICES = 32u;
constant float FIRST_FROXEL_SLICE_DEPTH = 1.0;
constant float FROXEL_SLICES_PER_OCTAVE = 3.0;
constant uint FROXEL_LIGHT_CAPACITY = 127u;
constant uint BEND_WORKGROUP_SIZE = 64u;
constant float QTANGENT_SMALLEST_W = 0.00003051851;
constant uint SHADOW_MASK_TILE_SIZE = 8u;
constant uint SHADOW_MASK_LIGHTS = 59u;
constant uint SHADOW_EDGE_LIGHTS = 7u;
constant uint SAMPLE_COUNT = 4u;
constant uint ALL_SAMPLES = 15u;
constant uint SLICES_PER_CLASS = 256u;
constant type_4 SAMPLE_OFFSETS = type_4 {{metal::float2(0.375, 0.125), metal::float2(0.875, 0.375), metal::float2(0.125, 0.625), metal::float2(0.625, 0.875)}};
constant metal::float2 FROXEL_GRID_SIZE = metal::float2(32.0, 18.0);
constant float UNBOUNDED_RAY_DISTANCE = 340282350000000000000000000000000000000.0;
constant float SHADOW_RAY_OFFSET_SCALE = 0.000015258789;
constant float SHADOW_RAY_PIXEL_OFFSET = 0.03125;
constant float DYNAMIC_TLAS_BOUNDS_MARGIN_SCALE = 0.000015258789;
constant float INTERPOLATED_TURN_MARGIN = 0.034899496;
constant float AGREEING_TRIANGLE_TURN_MARGIN = 0.034899496;
constant float TRIANGLE_TURN_MARGIN = 0.17364818;
constant float NORMALS_AGREE = 0.9986295;
constant uint VERDICT_HIDDEN_SHIFT = 8u;
constant uint VERDICT_TRACED_SHIFT = 16u;
constant uint EDGE_LIGHTS = 127u;
constant bool FROXEL_LIGHTS = true;

metal::float3 camera_relative_position(
    metal::int3 anchor,
    metal::float3 offset,
    metal::int3 camera_anchor,
    metal::float3 camera_offset,
    metal::float3 local_position
) {
    metal::float3 anchor_position = static_cast<metal::float3>(as_type<metal::int3>(as_type<metal::uint3>(anchor) - as_type<metal::uint3>(camera_anchor))) * ANCHOR_CELL_SIZE;
    return (anchor_position + (offset - camera_offset)) + local_position;
}

metal::float3 position_at(
    metal::float4x4 view_proj,
    metal::float2 point,
    float depth,
    metal::float2 size
) {
    metal::float2 device_ = metal::float2(((point.x / size.x) * 2.0) - 1.0, 1.0 - ((point.y / size.y) * 2.0));
    metal::float4x4 rows = metal::transpose(view_proj);
    float view_depth_1 = view_proj[3].z / depth;
    metal::float3 right = rows[0].xyz / metal::float3(metal::dot(rows[0].xyz, rows[0].xyz));
    metal::float3 up = rows[1].xyz / metal::float3(metal::dot(rows[1].xyz, rows[1].xyz));
    return view_depth_1 * ((rows[3].xyz + (device_.x * right)) + (device_.y * up));
}

bool segment_meets_box(
    metal::float3 origin,
    metal::float3 direction,
    float start_distance,
    float end_distance,
    metal::float3 box_min,
    metal::float3 box_max
) {
    metal::float3 centre = 0.5 * (box_min + box_max);
    metal::float3 half_extent = 0.5 * (box_max - box_min);
    metal::float3 from_centre = origin - centre;
    float cut_end_distance = metal::min(end_distance, metal::length(from_centre) + metal::length(half_extent));
    if (cut_end_distance < start_distance) {
        return false;
    }
    float half_length = 0.5 * (cut_end_distance - start_distance);
    metal::float3 midpoint = from_centre + (direction * (start_distance + half_length));
    metal::float3 half_segment = direction * half_length;
    metal::float3 reach = metal::abs(half_segment);
    if (metal::any(metal::abs(midpoint) > (half_extent + reach))) {
        return false;
    }
    metal::float3 crossed_reach = metal::float3((half_extent.y * reach.z) + (half_extent.z * reach.y), (half_extent.x * reach.z) + (half_extent.z * reach.x), (half_extent.x * reach.y) + (half_extent.y * reach.x));
    return !(metal::any(metal::abs(metal::cross(midpoint, half_segment)) > crossed_reach));
}

metal::float3 point_light_position(
    LightData light,
    metal::int3 camera_anchor_1,
    metal::float3 camera_offset_1
) {
    metal::float3 _e7 = camera_relative_position(light.anchor, light.offset, camera_anchor_1, camera_offset_1, metal::float3(0.0));
    return _e7;
}

metal::float3 octahedral_decode(
    metal::float2 coordinates
) {
    float z = (1.0 - metal::abs(coordinates.x)) - metal::abs(coordinates.y);
    metal::float2 sign = metal::select(metal::float2(-1.0), metal::float2(1.0), coordinates >= metal::float2(0.0));
    metal::float2 unfolded = (z >= 0.0) ? coordinates : ((metal::float2(1.0) - metal::abs(coordinates.yx)) * sign);
    return metal::normalize(metal::float3(unfolded, z));
}

uint froxel_index(
    metal::uint2 rectangle,
    uint slice
) {
    return (((rectangle.y * FROXEL_COLUMNS) + rectangle.x) * FROXEL_SLICES) + slice;
}

uint naga_f2u32(float value) {
    return static_cast<uint>(metal::clamp(value, 0.0, 4294967000.0));
}

uint froxel_slice(
    float depth_1
) {
    float slice_2 = metal::floor(metal::log2(depth_1 / FIRST_FROXEL_SLICE_DEPTH) * FROXEL_SLICES_PER_OCTAVE) + 1.0;
    return naga_f2u32(metal::clamp(slice_2, 0.0, 31.0));
}

metal::uint2 naga_f2u32(metal::float2 value) {
    return static_cast<metal::uint2>(metal::clamp(value, 0.0, 4294967000.0));
}

uint froxel_at(
    metal::float2 point_1,
    metal::float2 size_1,
    float view_depth
) {
    metal::float2 rectangle_1 = metal::clamp(metal::floor(metal::float2(point_1.x / size_1.x, 1.0 - (point_1.y / size_1.y)) * FROXEL_GRID_SIZE), metal::float2(0.0), metal::float2(31.0, 17.0));
    uint _e22 = froxel_slice(view_depth);
    uint _e23 = froxel_index(naga_f2u32(rectangle_1), _e22);
    return _e23;
}

RayIntersection ray_query_get_intersection_true(metal::raytracing::intersection_query<metal::raytracing::instancing, metal::raytracing::triangle_data> intersector
) {
    RayIntersection intersection = RayIntersection {};
    metal::raytracing::intersection_type ty = intersector.get_committed_intersection_type();
    if (ty == metal::raytracing::intersection_type::triangle) {
        intersection.kind = 1;
        intersection.front_face = intersector.is_committed_triangle_front_facing();
    } else if (ty == metal::raytracing::intersection_type::bounding_box) {
        intersection.kind = 2;
    }
    if (ty != metal::raytracing::intersection_type::none) {
        intersection.t = intersector.get_committed_distance();
    }
    return intersection;
}
RayIntersection first_intersection(
    uint flags,
    metal::float3 origin_1,
    metal::float3 direction_1,
    float start_distance_1,
    float end_distance_1,
    metal::raytracing::instance_acceleration_structure static_tlas,
    metal::raytracing::instance_acceleration_structure dynamic_tlas,
    constant CameraUniform& camera
) {
    metal::raytracing::intersection_query<metal::raytracing::instancing, metal::raytracing::triangle_data> query = {};
    bool local = {};
    bool local_1 = {};
    RayDesc _e10 = RayDesc {1u | flags, 255u, start_distance_1, end_distance_1, origin_1, direction_1};
    {
        RayDesc desc = _e10;
        metal::raytracing::intersection_params params;
        params.set_opacity_cull_mode(metal::raytracing::opacity_cull_mode::none);
        params.force_opacity(metal::raytracing::forced_opacity::opaque);
        params.accept_any_intersection((desc.flags & 4) != 0);
        metal::raytracing::ray ray = metal::raytracing::ray(desc.origin, desc.dir, desc.tmin, desc.tmax);
        query.reset(ray,static_tlas, desc.cull_mask, params);
    }
    bool _e11 = false;
    _e11 = query.next();
    RayIntersection static_hit = ray_query_get_intersection_true(query);
    bool found = static_hit.kind != 0u;
    uint _e18 = camera.dynamic_tlas_entries;
    if (!((_e18 == 0u))) {
        if (found) {
            local_1 = (flags & 4u) != 0u;
        } else {
            local_1 = false;
        }
        bool _e31 = local_1;
        local = _e31;
    } else {
        local = true;
    }
    bool _e33 = local;
    if (_e33) {
        return static_hit;
    }
    float dynamic_end_distance = found ? static_hit.t : end_distance_1;
    metal::float3 bounds_min = camera.dynamic_tlas_bounds_min;
    metal::float3 bounds_max = camera.dynamic_tlas_bounds_max;
    metal::float3 extent = metal::max(metal::abs(origin_1), metal::max(metal::abs(bounds_min), metal::abs(bounds_max)));
    float margin = DYNAMIC_TLAS_BOUNDS_MARGIN_SCALE * metal::max(metal::max(extent.x, extent.y), metal::max(extent.z, 1.0));
    bool _e60 = segment_meets_box(origin_1, direction_1, start_distance_1, dynamic_end_distance, bounds_min - metal::float3(margin), bounds_max + metal::float3(margin));
    if (!(_e60)) {
        return static_hit;
    }
    RayDesc _e66 = RayDesc {1u | flags, 255u, start_distance_1, dynamic_end_distance, origin_1, direction_1};
    {
        RayDesc desc = _e66;
        metal::raytracing::intersection_params params;
        params.set_opacity_cull_mode(metal::raytracing::opacity_cull_mode::none);
        params.force_opacity(metal::raytracing::forced_opacity::opaque);
        params.accept_any_intersection((desc.flags & 4) != 0);
        metal::raytracing::ray ray = metal::raytracing::ray(desc.origin, desc.dir, desc.tmin, desc.tmax);
        query.reset(ray,dynamic_tlas, desc.cull_mask, params);
    }
    bool _e67 = false;
    _e67 = query.next();
    RayIntersection dynamic_hit = ray_query_get_intersection_true(query);
    if (dynamic_hit.kind == 0u) {
        return static_hit;
    }
    return dynamic_hit;
}

bool shadowed(
    metal::float3 position,
    float pixel_width,
    metal::float3 triangle_normal,
    metal::float3 interpolated_normal,
    metal::float3 direction_2,
    float distance,
    metal::raytracing::instance_acceleration_structure static_tlas,
    metal::raytracing::instance_acceleration_structure dynamic_tlas,
    constant CameraUniform& camera
) {
    bool local_2 = {};
    bool local_3 = {};
    uint flags_1 = {};
    float start_distance_2 = 0.0;
    bool hidden = false;
    bool local_4 = {};
    bool local_5 = {};
    bool local_6 = {};
    bool local_7 = {};
    bool local_8 = {};
    metal::int3 _e8 = camera.anchor;
    metal::float3 _e11 = camera.offset;
    metal::int3 _e14 = camera.tlas_anchor;
    metal::float3 _e17 = camera_relative_position(_e8, _e11, _e14, metal::float3(0.0), position);
    metal::float3 extent_1 = metal::max(metal::abs(_e17), metal::abs(position));
    float offset_1 = metal::max(SHADOW_RAY_OFFSET_SCALE * metal::max(metal::max(extent_1.x, extent_1.y), metal::max(extent_1.z, 1.0)), SHADOW_RAY_PIXEL_OFFSET * pixel_width);
    float facing_1 = metal::dot(triangle_normal, direction_2);
    if (!((facing_1 >= TRIANGLE_TURN_MARGIN))) {
        if (facing_1 >= AGREEING_TRIANGLE_TURN_MARGIN) {
            local_3 = metal::dot(triangle_normal, interpolated_normal) >= NORMALS_AGREE;
        } else {
            local_3 = false;
        }
        bool _e47 = local_3;
        local_2 = _e47;
    } else {
        local_2 = true;
    }
    bool leaves_surface = local_2;
    metal::float3 origin_2 = _e17 + (triangle_normal * (leaves_surface ? offset_1 : -(offset_1)));
    flags_1 = leaves_surface ? 4u : 0u;
    while(true) {
        uint _e62 = flags_1;
        float _e63 = start_distance_2;
        RayIntersection _e64 = first_intersection(_e62, origin_2, direction_2, _e63, distance, static_tlas, dynamic_tlas, camera);
        bool found_1 = _e64.kind != 0u;
        uint _e68 = flags_1;
        bool any_hit = (_e68 & 4u) != 0u;
        if (!(any_hit)) {
            local_4 = !(found_1);
        } else {
            local_4 = true;
        }
        bool _e78 = local_4;
        if (!(_e78)) {
            local_5 = _e64.front_face;
        } else {
            local_5 = true;
        }
        bool _e84 = local_5;
        if (!(_e84)) {
            local_6 = (_e64.t + offset_1) >= distance;
        } else {
            local_6 = true;
        }
        bool _e92 = local_6;
        if (_e92) {
            if (found_1) {
                if (!(any_hit)) {
                    local_8 = _e64.front_face;
                } else {
                    local_8 = true;
                }
                bool _e100 = local_8;
                local_7 = _e100;
            } else {
                local_7 = false;
            }
            bool _e102 = local_7;
            hidden = _e102;
            break;
        }
        flags_1 = 4u;
        start_distance_2 = _e64.t + offset_1;
    }
    bool _e106 = hidden;
    return _e106;
}

metal::float2 sample_offset(
    uint sample
) {
    return (sample < 2u) ? ((sample == 0u) ? metal::float2(0.375, 0.125) : metal::float2(0.875, 0.375)) : ((sample == 2u) ? metal::float2(0.125, 0.625) : metal::float2(0.625, 0.875));
}

metal::uint2 with_bit(
    metal::uint2 bits,
    uint bit
) {
    uint shifted = 1u << (bit & 31u);
    return bits | ((bit >= 32u) ? metal::uint2(0u, shifted) : metal::uint2(shifted, 0u));
}

SampleSurface sample_surface(
    metal::int2 pixel,
    uint sample_1,
    float depth_2,
    metal::float2 size_2,
    constant CameraUniform& camera,
    metal::texture2d_ms<float, metal::access::read> surface_normals
) {
    metal::float2 _e5 = sample_offset(sample_1);
    metal::float2 point_2 = static_cast<metal::float2>(pixel) + _e5;
    metal::float4x4 _e9 = camera.view_proj;
    metal::float3 _e10 = position_at(_e9, point_2, depth_2, size_2);
    metal::float4 _e12 = surface_normals.read(metal::uint2(pixel), sample_1);
    metal::float4 normals = (_e12 * 2.0) - metal::float4(1.0);
    metal::float4x4 _e20 = camera.view_proj;
    metal::float3 _e25 = position_at(_e20, point_2 + metal::float2(1.0, 0.0), depth_2, size_2);
    metal::float3 _e28 = octahedral_decode(normals.xy);
    metal::float3 _e30 = octahedral_decode(normals.zw);
    return SampleSurface {_e10, metal::distance(_e10, _e25), _e28, _e30};
}

SurfaceWord surface_word(
    metal::int2 pixel_1,
    uint sample_2,
    float depth_3,
    metal::float2 size_3,
    uint other_samples,
    metal::raytracing::instance_acceleration_structure static_tlas,
    metal::raytracing::instance_acceleration_structure dynamic_tlas,
    constant CameraUniform& camera,
    constant LightUniform& light_1,
    device type_10 const& lights,
    device type_12 const& froxel_lists,
    metal::depth2d_ms<float, metal::access::read> depth_target,
    metal::texture2d_ms<float, metal::access::read> surface_normals,
    device metal::atomic_uint& shadow_mask_lights_overflowed,
    constant _mslBufferSizes& _buffer_sizes
) {
    metal::uint2 hidden_1 = metal::uint2(0u);
    uint traced = 0u;
    uint slice_1 = 0u;
    uint overflowed = 0u;
    uint froxel = 0u;
    uint point_light_count = {};
    uint entry = 0u;
    metal::float3 direction_3 = {};
    float distance_1 = {};
    SampleSurface traced_surface = {};
    uint index = {};
    bool facing = {};
    uint other_sample = {};
    bool local_9 = {};
    if (depth_3 > 0.0) {
        SampleSurface _e14 = sample_surface(pixel_1, sample_2, depth_3, size_3, camera, surface_normals);
        metal::float3 position_1 = _e14.position;
        metal::float3 triangle_normal_1 = _e14.triangle_normal;
        metal::float3 interpolated_normal_1 = _e14.interpolated_normal;
        uint _e24 = light_1.point_light_count;
        point_light_count = _e24;
        if (FROXEL_LIGHTS) {
            float _e31 = camera.view_proj[3].z;
            float view_depth_2 = _e31 / depth_3;
            uint _e37 = froxel_at(static_cast<metal::float2>(pixel_1) + metal::float2(0.5), size_3, view_depth_2);
            froxel = _e37;
            uint _e38 = froxel;
            slice_1 = _e38 % FROXEL_SLICES;
            uint _e42 = froxel;
            uint _e45 = froxel_lists[_e42].light_count;
            point_light_count = _e45;
        }
        uint _e48 = light_1.directional_light_count;
        uint _e49 = point_light_count;
        uint light_count_1 = _e48 + _e49;
        bool loop_init = true;
        while(true) {
            if (!loop_init) {
                uint _e203 = entry;
                entry = _e203 + 1u;
            }
            loop_init = false;
            uint _e53 = entry;
            if (_e53 < light_count_1) {
            } else {
                break;
            }
            {
                direction_3 = metal::float3 {};
                distance_1 = float {};
                traced_surface = _e14;
                uint _e60 = entry;
                uint _e63 = light_1.directional_light_count;
                if (_e60 < _e63) {
                    uint _e65 = entry;
                    if (_e65 >= SHADOW_MASK_LIGHTS) {
                        uint _e69 = overflowed;
                        overflowed = _e69 + 1u;
                        continue;
                    }
                    uint _e72 = entry;
                    metal::float3 _e75 = lights[_e72].offset;
                    direction_3 = metal::normalize(-(_e75));
                    distance_1 = UNBOUNDED_RAY_DISTANCE;
                    metal::float3 _e79 = direction_3;
                    if (!((metal::dot(interpolated_normal_1, _e79) > -0.034899496))) {
                        metal::uint2 _e84 = hidden_1;
                        uint _e85 = entry;
                        metal::uint2 _e86 = with_bit(_e84, _e85);
                        hidden_1 = _e86;
                        continue;
                    }
                    uint _e87 = entry;
                    if (_e87 < SHADOW_EDGE_LIGHTS) {
                        uint _e90 = traced;
                        uint _e92 = entry;
                        traced = _e90 | (1u << _e92);
                    }
                } else {
                    uint _e95 = entry;
                    index = _e95;
                    if (FROXEL_LIGHTS) {
                        uint _e99 = froxel;
                        uint _e102 = entry;
                        uint _e105 = light_1.directional_light_count;
                        uint _e108 = froxel_lists[_e99].lights.inner[_e102 - _e105];
                        index = _e108;
                    }
                    uint _e110 = index;
                    LightData point_3 = lights[_e110];
                    metal::int3 _e115 = camera.anchor;
                    metal::float3 _e118 = camera.offset;
                    metal::float3 _e119 = point_light_position(point_3, _e115, _e118);
                    metal::float3 to_light = _e119 - position_1;
                    float distance_squared = metal::dot(to_light, to_light);
                    if (distance_squared >= (point_3.range * point_3.range)) {
                        continue;
                    }
                    uint _e126 = entry;
                    if (_e126 >= SHADOW_MASK_LIGHTS) {
                        uint _e130 = overflowed;
                        overflowed = _e130 + 1u;
                        continue;
                    }
                    distance_1 = metal::sqrt(distance_squared);
                    float _e135 = distance_1;
                    if (metal::dot(interpolated_normal_1, to_light) <= (-0.034899496 * _e135)) {
                        facing = false;
                        other_sample = 0u;
                        bool loop_init_1 = true;
                        while(true) {
                            if (!loop_init_1) {
                                uint _e172 = other_sample;
                                other_sample = _e172 + 1u;
                            }
                            loop_init_1 = false;
                            uint _e142 = other_sample;
                            if (_e142 < SAMPLE_COUNT) {
                            } else {
                                break;
                            }
                            {
                                bool _e145 = facing;
                                if (!(_e145)) {
                                    uint _e149 = other_sample;
                                    local_9 = ((other_samples >> _e149) & 1u) == 0u;
                                } else {
                                    local_9 = true;
                                }
                                bool _e156 = local_9;
                                if (_e156) {
                                    continue;
                                }
                                uint _e157 = other_sample;
                                uint _e159 = other_sample;
                                float _e160 = depth_target.read(metal::uint2(pixel_1), _e159);
                                SampleSurface _e161 = sample_surface(pixel_1, _e157, _e160, size_3, camera, surface_normals);
                                metal::float3 other_to_light = _e119 - _e161.position;
                                float other_distance = metal::length(other_to_light);
                                if (metal::dot(_e161.interpolated_normal, other_to_light) > (-0.034899496 * other_distance)) {
                                    facing = true;
                                    traced_surface = _e161;
                                    distance_1 = other_distance;
                                }
                            }
                        }
                        bool _e174 = facing;
                        if (!(_e174)) {
                            metal::uint2 _e176 = hidden_1;
                            uint _e177 = entry;
                            metal::uint2 _e178 = with_bit(_e176, _e177);
                            hidden_1 = _e178;
                            continue;
                        }
                        metal::float3 _e180 = traced_surface.position;
                        float _e182 = distance_1;
                        direction_3 = (_e119 - _e180) / metal::float3(_e182);
                    } else {
                        float _e185 = distance_1;
                        direction_3 = to_light / metal::float3(_e185);
                    }
                }
                metal::float3 _e189 = traced_surface.position;
                float _e191 = traced_surface.pixel_width;
                metal::float3 _e193 = traced_surface.triangle_normal;
                metal::float3 _e195 = traced_surface.interpolated_normal;
                metal::float3 _e196 = direction_3;
                float _e197 = distance_1;
                bool _e198 = shadowed(_e189, _e191, _e193, _e195, _e196, _e197, static_tlas, dynamic_tlas, camera);
                if (_e198) {
                    metal::uint2 _e199 = hidden_1;
                    uint _e200 = entry;
                    metal::uint2 _e201 = with_bit(_e199, _e200);
                    hidden_1 = _e201;
                }
            }
        }
        uint _e205 = overflowed;
        if (_e205 > 0u) {
            uint _e209 = overflowed;
            uint _e210 = metal::atomic_fetch_add_explicit(&shadow_mask_lights_overflowed, _e209, metal::memory_order_relaxed);
        }
    }
    uint _e213 = hidden_1.x;
    uint _e215 = hidden_1.y;
    uint _e216 = slice_1;
    uint _e220 = traced;
    return SurfaceWord {metal::uint2(_e213, _e215 | (_e216 << 27u)), _e220};
}

uint first_sample_of(
    metal::float4 values,
    float value
) {
    return (values.x == value) ? 0u : ((values.y == value) ? 1u : ((values.z == value) ? 2u : 3u));
}

uint other_nearest_samples(
    metal::float4 depths,
    uint nearest,
    uint second_samples_1
) {
    metal::uint4 shown = metal::select(metal::uint4(0u), metal::uint4(1u, 2u, 4u, 8u), depths > metal::float4(0.0));
    return ((((shown.x | shown.y) | shown.z) | shown.w) & ~(second_samples_1)) & ~((1u << nearest));
}

uint at_every_sample(
    uint hidden_2,
    uint light_count
) {
    uint sample_verdicts = 0u;
    uint index_1 = 0u;
    bool loop_init_2 = true;
    while(true) {
        if (!loop_init_2) {
            uint _e21 = index_1;
            index_1 = _e21 + 1u;
        }
        loop_init_2 = false;
        uint _e6 = index_1;
        if (_e6 < light_count) {
        } else {
            break;
        }
        {
            uint _e8 = sample_verdicts;
            uint _e9 = index_1;
            uint _e15 = index_1;
            sample_verdicts = _e8 | (((hidden_2 >> _e9) & 1u) * (ALL_SAMPLES << (SAMPLE_COUNT * _e15)));
        }
    }
    uint _e23 = sample_verdicts;
    return _e23;
}

struct build_shadow_maskInput {
};
[[max_total_threads_per_threadgroup(64)]] kernel void build_shadow_mask(
  metal::uint3 invocation [[thread_position_in_grid]]
, metal::raytracing::instance_acceleration_structure static_tlas [[buffer(0)]]
, metal::raytracing::instance_acceleration_structure dynamic_tlas [[buffer(1)]]
, constant CameraUniform& camera [[buffer(2)]]
, constant LightUniform& light_1 [[buffer(3)]]
, device type_10 const& lights [[buffer(4)]]
, device type_12 const& froxel_lists [[buffer(5)]]
, metal::depth2d_ms<float, metal::access::read> depth_target [[texture(0)]]
, metal::texture2d_ms<float, metal::access::read> surface_normals [[texture(1)]]
, metal::texture2d_array<uint, metal::access::write> shadow_mask [[texture(2)]]
, metal::texture2d<uint, metal::access::write> shadow_mask_samples [[texture(3)]]
, device metal::atomic_uint& shadow_mask_lights_overflowed [[buffer(6)]]
, device PixelList& second_surfaces [[buffer(7)]]
, device type_19& second_surface_dispatch [[buffer(8)]]
, device type_18& directional_verdicts [[buffer(9)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(12)]]
) {
    uint second_samples = 0u;
    metal::int2 size_4 = static_cast<metal::int2>(metal::uint2(depth_target.get_width(), depth_target.get_height()));
    metal::int2 pixel_2 = static_cast<metal::int2>(invocation.xy);
    if (metal::any(pixel_2 >= size_4)) {
        return;
    }
    metal::float2 target_size = static_cast<metal::float2>(size_4);
    float _e11 = depth_target.read(metal::uint2(pixel_2), 0);
    float _e14 = depth_target.read(metal::uint2(pixel_2), 1);
    float _e17 = depth_target.read(metal::uint2(pixel_2), 2);
    float _e20 = depth_target.read(metal::uint2(pixel_2), 3);
    metal::float4 depths_1 = metal::float4(_e11, _e14, _e17, _e20);
    float depth_4 = metal::max(metal::max(depths_1.x, depths_1.y), metal::max(depths_1.z, depths_1.w));
    uint _e29 = first_sample_of(depths_1, depth_4);
    if (depth_4 > 0.0) {
        SampleSurface _e34 = sample_surface(pixel_2, _e29, depth_4, target_size, camera, surface_normals);
        metal::float3 position_2 = _e34.position;
        metal::float3 plane_normal = _e34.triangle_normal;
        metal::float4x4 _e39 = camera.view_proj;
        metal::float3 _e46 = position_at(_e39, static_cast<metal::float2>(pixel_2) + metal::float2(0.375, 0.125), depths_1.x, target_size);
        metal::float4x4 _e51 = camera.view_proj;
        metal::float3 _e58 = position_at(_e51, static_cast<metal::float2>(pixel_2) + metal::float2(0.875, 0.375), depths_1.y, target_size);
        metal::float4x4 _e63 = camera.view_proj;
        metal::float3 _e70 = position_at(_e63, static_cast<metal::float2>(pixel_2) + metal::float2(0.125, 0.625), depths_1.z, target_size);
        metal::float4x4 _e75 = camera.view_proj;
        metal::float3 _e82 = position_at(_e75, static_cast<metal::float2>(pixel_2) + metal::float2(0.625, 0.875), depths_1.w, target_size);
        metal::float4 off_plane = metal::float4(metal::dot(_e46 - position_2, plane_normal), metal::dot(_e58 - position_2, plane_normal), metal::dot(_e70 - position_2, plane_normal), metal::dot(_e82 - position_2, plane_normal));
        metal::bool4 second = (metal::abs(off_plane) > metal::float4(_e34.pixel_width)) & (depths_1 > metal::float4(0.0));
        metal::uint4 bits_1 = metal::select(metal::uint4(0u), metal::uint4(1u, 2u, 4u, 8u), second);
        second_samples = ((bits_1.x | bits_1.y) | bits_1.z) | bits_1.w;
        uint _e109 = second_samples;
        if (_e109 != 0u) {
            metal::float4 candidates = metal::select(metal::float4(2.0), depths_1, second);
            float second_depth = metal::min(metal::min(candidates.x, candidates.y), metal::min(candidates.z, candidates.w));
            uint _e122 = first_sample_of(candidates, second_depth);
            uint _e126 = metal::atomic_fetch_add_explicit(&second_surfaces.count, 1u, metal::memory_order_relaxed);
            second_surfaces.entries[_e126] = (invocation.x | (invocation.y << 16u)) | (_e122 << 30u);
            if ((_e126 % 64u) == 0u) {
                uint groups = (_e126 / 64u) + 1u;
                uint _e149 = metal::atomic_fetch_max_explicit(&second_surface_dispatch.inner[0], metal::min(groups, MAX_WORKGROUPS_PER_DIMENSION), metal::memory_order_relaxed);
                uint _e158 = metal::atomic_fetch_max_explicit(&second_surface_dispatch.inner[1], ((groups + MAX_WORKGROUPS_PER_DIMENSION) - 1u) / MAX_WORKGROUPS_PER_DIMENSION, metal::memory_order_relaxed);
                metal::atomic_store_explicit(&second_surface_dispatch.inner[2], 1u, metal::memory_order_relaxed);
            }
        }
    }
    uint _e162 = second_samples;
    uint _e163 = other_nearest_samples(depths_1, _e29, _e162);
    SurfaceWord _e164 = surface_word(pixel_2, _e29, depth_4, target_size, _e163, static_tlas, dynamic_tlas, camera, light_1, lights, froxel_lists, depth_target, surface_normals, shadow_mask_lights_overflowed, _buffer_sizes);
    shadow_mask.write(metal::uint4(_e164.word, 0u, 0u), metal::uint2(pixel_2), 0);
    uint _e173 = light_1.directional_light_count;
    uint edge_light_count = metal::min(_e173, SHADOW_EDGE_LIGHTS);
    uint hidden_3 = _e164.word.x & ((1u << edge_light_count) - 1u);
    uint _e183 = at_every_sample(hidden_3, edge_light_count);
    uint _e185 = second_samples;
    shadow_mask_samples.write(metal::uint4(_e185 | (_e183 << SAMPLE_COUNT), 0u, 0u, 0u), metal::uint2(pixel_2));
    uint _e201 = second_samples;
    directional_verdicts[(invocation.y * static_cast<uint>(size_4.x)) + invocation.x] = (_e201 | (hidden_3 << VERDICT_HIDDEN_SHIFT)) | (_e164.traced_directional_lights << VERDICT_TRACED_SHIFT);
    return;
}


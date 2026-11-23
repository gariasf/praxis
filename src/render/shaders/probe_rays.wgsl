const PROBE_GRID: vec3<u32> = vec3<u32>(32u, 8u, 32u);
const PROBE_SPACING_METRES: f32 = 4.0;
const PROBE_RAY_WORKGROUP_SIZE: u32 = 64u;
const GOLDEN_ANGLE: f32 = 2.399963229728653;
const EVERY_ENTRY: u32 = 0xffu;

const UNBOUNDED_RAY_DISTANCE: f32 = 3.40282347e38;
const SHADOW_RAY_OFFSET_SCALE: f32 = 1.52587890625e-5; // 2^-16
const DYNAMIC_TLAS_BOUNDS_MARGIN_SCALE: f32 = 1.52587890625e-5; // 2^-16

override RAYS_PER_PROBE: u32;
override RAY_LENGTH_METRES: f32;
override VERIFY_HITS: bool;

// Must stay equal to `first_intersection` of shadow_mask.wgsl.
fn first_intersection(flags: u32, mask: u32, origin: vec3<f32>, direction: vec3<f32>, start_distance: f32, end_distance: f32) -> RayIntersection {
    var query: ray_query;
    rayQueryInitialize(&query, static_tlas, RayDesc(RAY_FLAG_FORCE_OPAQUE | flags, mask, start_distance, end_distance, origin, direction));
    rayQueryProceed(&query);
    let static_hit = rayQueryGetCommittedIntersection(&query);
    let found = static_hit.kind != RAY_QUERY_INTERSECTION_NONE;
    if camera.dynamic_tlas_entries == 0u || (found && (flags & RAY_FLAG_TERMINATE_ON_FIRST_HIT) != 0u) {
        return static_hit;
    }
    let dynamic_end_distance = select(end_distance, static_hit.t, found);
    let bounds_min = camera.dynamic_tlas_bounds_min;
    let bounds_max = camera.dynamic_tlas_bounds_max;
    let extent = max(abs(origin), max(abs(bounds_min), abs(bounds_max)));
    let margin = DYNAMIC_TLAS_BOUNDS_MARGIN_SCALE * max(max(extent.x, extent.y), max(extent.z, 1.0));
    if !segment_meets_box(origin, direction, start_distance, dynamic_end_distance, bounds_min - margin, bounds_max + margin) {
        return static_hit;
    }
    rayQueryInitialize(&query, dynamic_tlas, RayDesc(RAY_FLAG_FORCE_OPAQUE | flags, mask, start_distance, dynamic_end_distance, origin, direction));
    rayQueryProceed(&query);
    let dynamic_hit = rayQueryGetCommittedIntersection(&query);
    if dynamic_hit.kind == RAY_QUERY_INTERSECTION_NONE {
        return static_hit;
    }
    return dynamic_hit;
}

fn uniform_direction(index: u32, count: u32) -> vec3<f32> {
    let height = 1.0 - (2.0 * f32(index) + 1.0) / f32(count);
    let radius = sqrt(max(1.0 - height * height, 0.0));
    let azimuth = GOLDEN_ANGLE * f32(index);
    return vec3<f32>(radius * cos(azimuth), height, radius * sin(azimuth));
}

fn vertex_position(mesh: u32, corner: u32) -> vec3<f32> {
    let position = vertices[meshes[mesh].first_vertex + indices[meshes[mesh].first_index + corner]].position;
    return vec3<f32>(position[0], position[1], position[2]);
}

fn hit_normal(hit: RayIntersection) -> vec3<f32> {
    // A TLAS entry's custom data is its Instance's slot.
    let slot = hit.instance_custom_data;
    let mesh = instances[slot].mesh;
    let first_corner = 3u * hit.primitive_index;
    let first = vertex_position(mesh, first_corner);
    let second = vertex_position(mesh, first_corner + 1u);
    let third = vertex_position(mesh, first_corner + 2u);
    return normalize(instances[slot].normal_matrix * cross(second - first, third - first));
}

fn hit_lies_on_its_triangle(hit: RayIntersection, position: vec3<f32>) -> bool {
    let mesh = instances[hit.instance_custom_data].mesh;
    let first_corner = 3u * hit.primitive_index;
    let first = vertex_position(mesh, first_corner);
    let second = vertex_position(mesh, first_corner + 1u);
    let third = vertex_position(mesh, first_corner + 2u);
    let on_triangle = first
        + hit.barycentrics.x * (second - first)
        + hit.barycentrics.y * (third - first);
    let placed = hit.object_to_world * vec4<f32>(on_triangle, 1.0);
    return distance(placed, position) < 0.01 + 1e-4 * hit.t;
}

@compute @workgroup_size(PROBE_RAY_WORKGROUP_SIZE)
fn trace_probe_rays(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let ray = invocation.x;
    let probe = ray / RAYS_PER_PROBE;
    let cell = vec3<u32>(
        probe % PROBE_GRID.x,
        (probe / PROBE_GRID.x) % PROBE_GRID.y,
        probe / (PROBE_GRID.x * PROBE_GRID.y)
    );
    let from_camera = (vec3<f32>(cell) - 0.5 * vec3<f32>(PROBE_GRID - 1u)) * PROBE_SPACING_METRES;
    let origin = camera_relative_position(
        camera.anchor,
        camera.offset,
        camera.tlas_anchor,
        vec3<f32>(0.0),
        from_camera
    );
    let direction = uniform_direction(ray % RAYS_PER_PROBE, RAYS_PER_PROBE);
    let hit = first_intersection(0u, EVERY_ENTRY, origin, direction, 0.0, RAY_LENGTH_METRES);
    if hit.kind == RAY_QUERY_INTERSECTION_NONE {
        probe_ray_results[ray] = 0u;
        return;
    }
    let normal = hit_normal(hit);
    let facing = select(normal, -normal, dot(normal, direction) > 0.0);
    let position = origin + direction * hit.t;
    let extent = abs(position);
    let offset = SHADOW_RAY_OFFSET_SCALE * max(max(extent.x, extent.y), max(extent.z, 1.0));
    var to_sun = vec3<f32>(0.0, 1.0, 0.0);
    if light.directional_light_count > 0u {
        to_sun = -normalize(lights[0].offset);
    }
    let sun_hit = first_intersection(
        RAY_FLAG_TERMINATE_ON_FIRST_HIT,
        EVERY_ENTRY,
        position + facing * offset,
        to_sun,
        0.0,
        UNBOUNDED_RAY_DISTANCE
    );
    let lit = sun_hit.kind == RAY_QUERY_INTERSECTION_NONE;
    let cosine = u32(saturate(dot(facing, to_sun)) * 255.0);
    var verified = 0u;
    if VERIFY_HITS {
        verified = u32(hit_lies_on_its_triangle(hit, position)) << 2u;
    }
    probe_ray_results[ray] = 1u | (u32(lit) << 1u) | verified | (cosine << 8u) | (u32(min(hit.t, 65535.0)) << 16u);
}

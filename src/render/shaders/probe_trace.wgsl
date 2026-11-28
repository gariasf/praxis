struct ProbeUpdateUniform {
    cascade_anchor: vec3<i32>,
    first_level: u32,
    cascade_offset: vec3<f32>,
}

// Past a Cascade cell's diagonal, the farthest a point a Probe lights lies from it.
const PROBE_DEPTH_LIMIT_METRES: f32 = 2.0 * PROBE_SPACING_METRES;

@compute @workgroup_size(PROBE_TRACE_WORKGROUP_SIZE)
fn trace_probe_rays(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let ray = invocation.x;
    let slot = invocation.y;
    let probes_per_level = PROBE_COUNTS.x * PROBE_COUNTS.z;
    let probe = vec3<u32>(
        slot % PROBE_COUNTS.x,
        probe_update.first_level + slot / probes_per_level,
        slot / PROBE_COUNTS.x % PROBE_COUNTS.z
    );
    let centred = (vec3<f32>(probe) - 0.5 * vec3<f32>(PROBE_COUNTS - 1u)) * PROBE_SPACING_METRES;
    let origin = camera_relative_position(
        probe_update.cascade_anchor,
        probe_update.cascade_offset,
        camera.tlas_anchor,
        vec3<f32>(0.0),
        centred
    );
    let direction = probe_ray_directions[ray].xyz;
    let hit = first_intersection(0u, 0xffu, origin, direction, 0.0, UNBOUNDED_RAY_DISTANCE);
    let missed = hit.kind == RAY_QUERY_INTERSECTION_NONE;
    let radiance = select(vec3<f32>(0.0), light.ambient_illuminance.rgb / PI, missed);
    let distance = select(min(hit.t, PROBE_DEPTH_LIMIT_METRES), PROBE_DEPTH_LIMIT_METRES, missed);
    probe_rays[slot * PROBE_RAYS + ray] = vec4<f32>(radiance, distance);
}

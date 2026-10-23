// A marker of the dynamic TLAS's build: one invocation that queries the
// dynamic TLAS and keeps what it found, so the GPU runs it after the TLAS's
// earlier work and before its next build, or after that build, and a
// timestamp at its end or its start brackets the build
// (src/render/acceleration_structures.rs).
@compute @workgroup_size(1)
fn mark_dynamic_tlas() {
    var query: ray_query;
    rayQueryInitialize(
        &query,
        dynamic_tlas,
        // Every instance's cull mask, and a ray too short to reach far.
        RayDesc(RAY_FLAG_NONE, 0xffu, 0.0, 1e-6, vec3<f32>(0.0), vec3<f32>(0.0, 1.0, 0.0))
    );
    rayQueryProceed(&query);
    marker_result = rayQueryGetCommittedIntersection(&query).kind;
}

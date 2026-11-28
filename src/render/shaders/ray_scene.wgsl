// Tracing the Ray scene (src/render/ray_scene.rs), shared by the passes that trace it.

// A directional light's shadow ray and a probe ray run to f32's largest
// distance, past anything a TLAS holds.
const UNBOUNDED_RAY_DISTANCE: f32 = 3.40282347e38;

// The box that holds the dynamic TLAS's triangles grows, for the test that
// skips it, by this share of the largest coordinate of its corners and the
// ray's origin, and of 1 m at least, the share a shadow ray's origin
// leaves its surface by. Placing the box on the CPU, placing a TLAS entry's
// triangles and intersecting them on the GPU, and the test each round a
// coordinate by a few times 2^-24 of the largest of them, by estimate, as
// no specification bounds them: well inside the margin's 256, so a ray that
// meets a triangle of the dynamic TLAS meets the box. The bend pass's
// rounding is in the box already (src/render/acceleration_structures.rs).
const DYNAMIC_TLAS_BOUNDS_MARGIN_SCALE: f32 = 1.52587890625e-5; // 2^-16

// The first triangle either TLAS holds, of an entry whose instance mask
// shares a bit with `mask`, from `start_distance` to `end_distance` (the
// ray's tmin and tmax) along the ray from `origin`, in the TLAS's frame,
// along `direction`, a unit vector: with
// RAY_FLAG_TERMINATE_ON_FIRST_HIT in `flags`, any one, and otherwise the
// nearest. Its kind is RAY_QUERY_INTERSECTION_NONE where there is none.
// It traces the static TLAS, then the dynamic TLAS unless it is empty, as a
// query of an empty TLAS still costs, a first hit has already been found, or
// the ray misses the box that holds the dynamic TLAS's triangles before it
// would reach the static hit, as a query starts at the TLAS's root however
// far the ray passes from its Instances.
// A shader's live ray queries hold registers that lower its occupancy
// [ref:nvidia-2022-rtx-best-practices-shadow-rays], so every ray of a
// surface goes through this one call. Its flags are then a value known
// only at run time, which costs a scene of few, short rays a little more
// than constant flags would, and saves scenes of many rays more
// (docs/performance.md).
fn first_intersection(flags: u32, mask: u32, origin: vec3<f32>, direction: vec3<f32>, start_distance: f32, end_distance: f32) -> RayIntersection {
    // Opaque, so one rayQueryProceed completes a traversal.
    var query: ray_query;
    rayQueryInitialize(&query, static_tlas, RayDesc(RAY_FLAG_FORCE_OPAQUE | flags, mask, start_distance, end_distance, origin, direction));
    rayQueryProceed(&query);
    let static_hit = rayQueryGetCommittedIntersection(&query);
    let found = static_hit.kind != RAY_QUERY_INTERSECTION_NONE;
    if camera.dynamic_tlas_entries == 0u || (found && (flags & RAY_FLAG_TERMINATE_ON_FIRST_HIT) != 0u) {
        return static_hit;
    }
    // Only a dynamic triangle nearer than the static one can replace it.
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

// The shadow mask pass (src/render/shadow_mask_pass.rs): an invocation per
// pixel traces one hard shadow ray toward each light that reaches the surface
// the pixel's nearest sample shows (ADR 0020), and keeps a bit per light that
// its ray found hidden, for the lit pass to read. The bits follow the pixel's
// lights in the order the lit pass sums them: the directional lights, then
// the point lights its Froxel lists, as many as SHADOW_MASK_LIGHTS, across
// the word's two halves; the second half's top bits hold the Froxel's slice.
// A light past them casts no shadow there, and is counted. The depth target
// holds both culling phases' Instances when it runs, and the surface normals
// what each sample shows; the surface's position is rebuilt from the
// sample's depth. Its bindings are declared from their bind groups'
// definitions (src/render/bind_groups.rs).

// A directional light's shadow ray runs to f32's largest distance, past
// anything a TLAS holds.
const UNBOUNDED_RAY_DISTANCE: f32 = 3.40282347e38;

// A shadow ray's origin leaves its surface along the normal by this share
// of the point's largest coordinate, relative to the camera or in the TLAS's
// frame, and of 1 m at least. Rebuilding the point from its depth, composing
// it into the TLAS's frame and intersecting the TLAS each round a coordinate
// by a few times 2^-24 of its size at most, well inside the offset's 256.
const SHADOW_RAY_OFFSET_SCALE: f32 = 1.52587890625e-5; // 2^-16

// The cosines, of a light's angle from a plane, past which a sample's normals
// face the light or turn from it: sin 2°, 5° and 10°. The surface normals'
// 8-bit octahedral coordinates hold each within 0.95° where the store rounds
// to the nearest step, as Metal's does, and about twice that where it may
// round to either (`octahedral_decode`). Within the first, the stored
// interpolated normal may turn from a light the lit pass's still faces; a
// triangle faces a light its ray then clears by the second where its
// normals agree, and by the third where the interpolated normal bends from
// it; short of them, its ray starts inside the surface.
const INTERPOLATED_TURN_MARGIN: f32 = 0.03489949670250097;
const AGREEING_TRIANGLE_TURN_MARGIN: f32 = 0.08715574274765817;
const TRIANGLE_TURN_MARGIN: f32 = 0.17364817766693033;

// The cosine within which a triangle's normal and the normal interpolated
// over it agree, as on a flat surface or a flat-shaded mesh: 3°.
const NORMALS_AGREE: f32 = 0.9986295347545738;

// Whether either TLAS holds geometry from `start_distance` to `end_distance`
// (the ray's tmin and tmax) along the ray from `origin`, in the TLAS's
// frame, along `direction`, a unit vector:
// traced against the static TLAS, then, where that finds nothing, against
// the dynamic TLAS unless it is empty: a query of an empty TLAS still costs.
fn hit_any(origin: vec3<f32>, direction: vec3<f32>, start_distance: f32, end_distance: f32) -> bool {
    // Opaque, so one rayQueryProceed completes a traversal, and any hit
    // shadows the point.
    let ray = RayDesc(
        RAY_FLAG_FORCE_OPAQUE | RAY_FLAG_TERMINATE_ON_FIRST_HIT,
        0xffu,
        start_distance,
        end_distance,
        origin,
        direction
    );
    var query: ray_query;
    rayQueryInitialize(&query, static_tlas, ray);
    rayQueryProceed(&query);
    let hidden = rayQueryGetCommittedIntersection(&query).kind != RAY_QUERY_INTERSECTION_NONE;
    if hidden || camera.dynamic_tlas_entries == 0u {
        return hidden;
    }
    rayQueryInitialize(&query, dynamic_tlas, ray);
    rayQueryProceed(&query);
    return rayQueryGetCommittedIntersection(&query).kind != RAY_QUERY_INTERSECTION_NONE;
}

// The nearest triangle either TLAS holds within `end_distance` along the ray from
// `origin`, in the TLAS's frame, along `direction`, a unit vector; its kind
// is RAY_QUERY_INTERSECTION_NONE where there is none.
fn nearest_hit(origin: vec3<f32>, direction: vec3<f32>, end_distance: f32) -> RayIntersection {
    var query: ray_query;
    rayQueryInitialize(&query, static_tlas, RayDesc(RAY_FLAG_FORCE_OPAQUE, 0xffu, 0.0, end_distance, origin, direction));
    rayQueryProceed(&query);
    let static_hit = rayQueryGetCommittedIntersection(&query);
    if camera.dynamic_tlas_entries == 0u {
        return static_hit;
    }
    // Only a dynamic triangle nearer than the static one can replace it.
    let dynamic_end_distance = select(end_distance, static_hit.t, static_hit.kind != RAY_QUERY_INTERSECTION_NONE);
    rayQueryInitialize(&query, dynamic_tlas, RayDesc(RAY_FLAG_FORCE_OPAQUE, 0xffu, 0.0, dynamic_end_distance, origin, direction));
    rayQueryProceed(&query);
    let dynamic_hit = rayQueryGetCommittedIntersection(&query);
    if dynamic_hit.kind == RAY_QUERY_INTERSECTION_NONE {
        return static_hit;
    }
    return dynamic_hit;
}

// Whether either TLAS holds geometry within `distance` of the surface point
// at `position`, relative to the camera, along `direction`, a unit vector:
// one hard shadow ray (ADR 0020), in the frame both TLAS share, whose origin
// is the corner of the camera's `tlas_anchor`, composed as an Instance's
// position is (ADR 0014). It starts off the surface along
// `triangle_normal` where the triangle faces the light. Near a smooth-shaded
// mesh's terminator, though, the triangle can turn from a light that
// `interpolated_normal` faces, and a ray leaving it would cross the mesh's
// own triangles: there it starts inside the surface, and passes the back of
// the first triangle it meets, where it leaves the mesh. A ray that enters
// a closed mesh meets a front face first, so whatever stands beyond still
// shadows the point.
fn shadowed(
    position: vec3<f32>,
    triangle_normal: vec3<f32>,
    interpolated_normal: vec3<f32>,
    direction: vec3<f32>,
    distance: f32
) -> bool {
    // The camera stands where an Instance does in this composition, and the
    // TLAS's frame, with no offset from its Anchor, where the camera does.
    let tlas_position = camera_relative_position(
        camera.anchor,
        camera.offset,
        camera.tlas_anchor,
        vec3<f32>(0.0),
        position
    );
    let extent = max(abs(tlas_position), abs(position));
    let offset = SHADOW_RAY_OFFSET_SCALE * max(max(extent.x, extent.y), max(extent.z, 1.0));
    let facing = dot(triangle_normal, direction);
    if facing >= TRIANGLE_TURN_MARGIN
        || (facing >= AGREEING_TRIANGLE_TURN_MARGIN
            && dot(triangle_normal, interpolated_normal) >= NORMALS_AGREE) {
        return hit_any(tlas_position + triangle_normal * offset, direction, 0.0, distance);
    }
    let origin = tlas_position - triangle_normal * offset;
    let exit = nearest_hit(origin, direction, distance);
    if exit.kind == RAY_QUERY_INTERSECTION_NONE {
        return false;
    }
    if exit.front_face || exit.t + offset >= distance {
        return exit.front_face;
    }
    return hit_any(origin, direction, exit.t + offset, distance);
}

// Where `sample` of a pixel lies, from the pixel's top left corner, in
// pixels: SAMPLE_OFFSETS, declared from Rust's, which a test checks the
// depth target's samples against (src/render/shadow_mask_pass.rs).
fn sample_offset(sample: u32) -> vec2<f32> {
    return select(
        select(SAMPLE_OFFSETS[3], SAMPLE_OFFSETS[2], sample == 2u),
        select(SAMPLE_OFFSETS[1], SAMPLE_OFFSETS[0], sample == 0u),
        sample < 2u
    );
}

// `bits` with bit `bit` of the word's 64 set, across its two halves.
fn with_bit(bits: vec2<u32>, bit: u32) -> vec2<u32> {
    let shifted = 1u << (bit & 31u);
    return bits | select(vec2<u32>(shifted, 0u), vec2<u32>(0u, shifted), bit >= 32u);
}

@compute @workgroup_size(SHADOW_MASK_TILE_SIZE, SHADOW_MASK_TILE_SIZE)
fn build_shadow_mask(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let size = vec2<i32>(textureDimensions(depth_target));
    let pixel = vec2<i32>(invocation.xy);
    if any(pixel >= size) {
        return;
    }
    // The four samples Rust asserts the depth target has (SAMPLE_COUNT),
    // loaded together: Metal's compiler does not unroll a loop over them.
    let depths = vec4<f32>(
        textureLoad(depth_target, pixel, 0),
        textureLoad(depth_target, pixel, 1),
        textureLoad(depth_target, pixel, 2),
        textureLoad(depth_target, pixel, 3)
    );
    // The nearest sample has the greatest depth (ADR 0015); the first such.
    let depth = max(max(depths.x, depths.y), max(depths.z, depths.w));
    let sample = select(
        select(select(3u, 2u, depths.z == depth), 1u, depths.y == depth),
        0u,
        depths.x == depth
    );
    var hidden = vec2<u32>(0u);
    var slice = 0u;
    // Depth 0 is the far plane, the background, which nothing shadows.
    if depth > 0.0 {
        let position = position_at(
            camera.view_proj,
            vec2<f32>(pixel) + sample_offset(sample),
            depth,
            vec2<f32>(size)
        );
        let normals = textureLoad(surface_normals, pixel, sample) * 2.0 - 1.0;
        let triangle_normal = octahedral_decode(normals.xy);
        let interpolated_normal = octahedral_decode(normals.zw);
        var overflowed = 0u;
        for (var index = 0u; index < light.directional_light_count; index++) {
            if index >= SHADOW_MASK_LIGHTS {
                overflowed++;
                continue;
            }
            // A light the interpolated normal turns from is hidden by the
            // surface itself, with no ray.
            let direction = normalize(-lights[index].offset);
            if dot(interpolated_normal, direction) <= -INTERPOLATED_TURN_MARGIN
                || shadowed(position, triangle_normal, interpolated_normal, direction, UNBOUNDED_RAY_DISTANCE) {
                hidden = with_bit(hidden, index);
            }
        }
        // The point lights the pixel's Froxel lists, in light order, or, with
        // FROXEL_LIGHTS off, every point light, as the lit pass sums them:
        // those within range of the surface.
        var froxel = 0u;
        var point_light_count = light.point_light_count;
        if FROXEL_LIGHTS {
            let view_depth = camera.view_proj[3].z / depth;
            froxel = froxel_at(vec2<f32>(pixel) + 0.5, vec2<f32>(size), view_depth);
            slice = froxel % FROXEL_SLICES;
            point_light_count = froxel_lists[froxel].light_count;
        }
        for (var entry = 0u; entry < point_light_count; entry++) {
            var index = light.directional_light_count + entry;
            if FROXEL_LIGHTS {
                index = froxel_lists[froxel].lights[entry];
            }
            let point = lights[index];
            let to_light = point_light_position(point, camera.anchor, camera.offset) - position;
            let distance_squared = dot(to_light, to_light);
            if distance_squared >= point.range * point.range {
                continue;
            }
            let bit = light.directional_light_count + entry;
            if bit >= SHADOW_MASK_LIGHTS {
                overflowed++;
                continue;
            }
            // Facing first, so a light at the surface, at no distance, is
            // hidden without dividing by it.
            let distance = sqrt(distance_squared);
            if dot(interpolated_normal, to_light) <= -INTERPOLATED_TURN_MARGIN * distance
                || shadowed(position, triangle_normal, interpolated_normal, to_light / distance, distance) {
                hidden = with_bit(hidden, bit);
            }
        }
        if overflowed > 0u {
            atomicAdd(&shadow_mask_lights_overflowed, overflowed);
        }
    }
    let slice_shift = SHADOW_MASK_LIGHTS - 32u;
    textureStore(shadow_mask, pixel, vec4<u32>(hidden.x, hidden.y | (slice << slice_shift), 0u, 0u));
}

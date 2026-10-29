// The shadow mask pass (src/render/shadow_mask_pass.rs): an invocation per
// pixel traces one hard shadow ray toward each light that reaches the surface
// the pixel's nearest sample shows (ADR 0020), and keeps a bit per light that
// its ray found hidden, for the lit pass to read. Where other samples of the
// pixel show another surface, it does the same from the farthest of them
// into the mask's second layer, and marks which samples show it, so each
// fragment of an edge pixel reads its own surface's bits. The bits follow
// the pixel's lights in the order the lit pass sums them: the directional
// lights, then the point lights its Froxel lists, as many as
// SHADOW_MASK_LIGHTS, across the word's two halves; the second half's top
// bits hold the Froxel's slice. A light past them casts no shadow there,
// and is counted. The depth target holds both culling phases' Instances
// when it runs, and the surface normals what each sample shows; the
// surface's position is rebuilt from the sample's depth. Its bindings are
// declared from their bind groups' definitions (src/render/bind_groups.rs).

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

// The word of the bits for the pixel's lights whose rays the surface that
// its `sample` shows, at `depth`, found hidden, with the slice of the
// Froxel they follow in the second half's top bits; none where `depth` is
// the background's.
fn surface_word(pixel: vec2<i32>, sample: u32, depth: f32, size: vec2<f32>) -> vec2<u32> {
    var hidden = vec2<u32>(0u);
    var slice = 0u;
    // Depth 0 is the far plane, the background, which nothing shadows.
    if depth > 0.0 {
        let position = position_at(
            camera.view_proj,
            vec2<f32>(pixel) + sample_offset(sample),
            depth,
            size
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
            froxel = froxel_at(vec2<f32>(pixel) + 0.5, size, view_depth);
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
    return vec2<u32>(hidden.x, hidden.y | (slice << slice_shift));
}

// The index of the first of four samples whose value in `values` is `value`.
fn first_sample_of(values: vec4<f32>, value: f32) -> u32 {
    return select(
        select(select(3u, 2u, values.z == value), 1u, values.y == value),
        0u,
        values.x == value
    );
}

// The second surfaces of the frame's pixels, listed for the second dispatch
// to trace: how many, then each one's pixel, its column in the low 16 bits
// and its row above them, and from bit 30 the sample to trace it from.
struct SecondSurfaces {
    count: atomic<u32>,
    entries: array<u32>,
}

// Each invocation lists its pixel's second surface, if any, and traces its
// nearest surface; `trace_second_surfaces` then traces the frame's list, an
// invocation each, as tile-based deferred renderers shade one sample of
// every pixel, then a packed list of the pixels that need the others: a ray
// traced for a few pixels of a SIMD group would hold up all its others
// [ref:lauritzen-2010-per-sample-scheduling].
@compute @workgroup_size(SHADOW_MASK_TILE_SIZE, SHADOW_MASK_TILE_SIZE)
fn build_shadow_mask(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let size = vec2<i32>(textureDimensions(depth_target));
    let pixel = vec2<i32>(invocation.xy);
    if any(pixel >= size) {
        return;
    }
    let target_size = vec2<f32>(size);
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
    let sample = first_sample_of(depths, depth);

    // The samples that show another surface than the nearest: a surface
    // discontinuity, found as deferred renderers find the pixels they shade
    // per sample, by comparing each sample's depth with the depth the
    // nearest sample's plane gives it [ref:lauritzen-2010-edge-detection]:
    // those off the plane of its triangle by more than a pixel's width at
    // its distance, other than the background's.
    var second_samples = 0u;
    if depth > 0.0 {
        let point = vec2<f32>(pixel) + sample_offset(sample);
        let position = position_at(camera.view_proj, point, depth, target_size);
        let pixel_width = distance(
            position,
            position_at(camera.view_proj, point + vec2<f32>(1.0, 0.0), depth, target_size)
        );
        let plane_normal = octahedral_decode(textureLoad(surface_normals, pixel, sample).xy * 2.0 - 1.0);
        let off_plane = vec4<f32>(
            dot(position_at(camera.view_proj, vec2<f32>(pixel) + SAMPLE_OFFSETS[0], depths.x, target_size) - position, plane_normal),
            dot(position_at(camera.view_proj, vec2<f32>(pixel) + SAMPLE_OFFSETS[1], depths.y, target_size) - position, plane_normal),
            dot(position_at(camera.view_proj, vec2<f32>(pixel) + SAMPLE_OFFSETS[2], depths.z, target_size) - position, plane_normal),
            dot(position_at(camera.view_proj, vec2<f32>(pixel) + SAMPLE_OFFSETS[3], depths.w, target_size) - position, plane_normal)
        );
        let second = abs(off_plane) > vec4<f32>(pixel_width) & depths > vec4<f32>(0.0);
        let bits = select(vec4<u32>(0u), vec4<u32>(1u, 2u, 4u, 8u), second);
        second_samples = bits.x | bits.y | bits.z | bits.w;
        // The farthest of them, the least depth, gives the second surface's
        // word: where they show more than one surface, theirs all.
        if second_samples != 0u {
            // Depth 2 lies past the near plane's 1, so no other is less.
            let candidates = select(vec4<f32>(2.0), depths, second);
            let second_depth = min(min(candidates.x, candidates.y), min(candidates.z, candidates.w));
            let second_sample = first_sample_of(candidates, second_depth);
            let entry = atomicAdd(&second_surfaces.count, 1u);
            second_surfaces.entries[entry] = invocation.x | (invocation.y << 16u) | (second_sample << 30u);
            // The entry that opens a workgroup's run of the second dispatch
            // adds that workgroup, in rows of MAX_WORKGROUPS_PER_DIMENSION.
            let group_size = SHADOW_MASK_TILE_SIZE * SHADOW_MASK_TILE_SIZE;
            if entry % group_size == 0u {
                let groups = entry / group_size + 1u;
                atomicMax(&second_surface_dispatch[0], min(groups, MAX_WORKGROUPS_PER_DIMENSION));
                atomicMax(
                    &second_surface_dispatch[1],
                    (groups + MAX_WORKGROUPS_PER_DIMENSION - 1u) / MAX_WORKGROUPS_PER_DIMENSION
                );
                atomicStore(&second_surface_dispatch[2], 1u);
            }
        }
    }
    textureStore(shadow_mask_samples, pixel, vec4<u32>(second_samples, 0u, 0u, 0u));
    textureStore(shadow_mask, pixel, 0, vec4<u32>(surface_word(pixel, sample, depth, target_size), 0u, 0u));
}

// How many second surfaces the frame lists, read once per workgroup: an
// atomic every invocation loads from one address serialises them.
var<workgroup> listed_count: u32;

// Traces the second surface `build_shadow_mask` listed at this invocation's
// entry, if any, into the shadow mask's second layer.
@compute @workgroup_size(SHADOW_MASK_TILE_SIZE * SHADOW_MASK_TILE_SIZE)
fn trace_second_surfaces(
    @builtin(workgroup_id) group: vec3<u32>,
    @builtin(num_workgroups) groups: vec3<u32>,
    @builtin(local_invocation_index) lane: u32
) {
    if lane == 0u {
        listed_count = atomicLoad(&second_surfaces.count);
    }
    let entry = (group.y * groups.x + group.x) * SHADOW_MASK_TILE_SIZE * SHADOW_MASK_TILE_SIZE + lane;
    if entry >= workgroupUniformLoad(&listed_count) {
        return;
    }
    let listed = second_surfaces.entries[entry];
    let pixel = vec2<i32>(vec2<u32>(listed & 0xffffu, (listed >> 16u) & 0x3fffu));
    let sample = listed >> 30u;
    let depth = textureLoad(depth_target, pixel, sample);
    let word = surface_word(pixel, sample, depth, vec2<f32>(textureDimensions(depth_target)));
    textureStore(shadow_mask, pixel, 1, vec4<u32>(word, 0u, 0u));
}

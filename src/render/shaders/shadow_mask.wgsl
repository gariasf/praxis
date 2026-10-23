// The shadow mask pass (src/render/shadow_mask_pass.rs): an invocation per
// pixel traces one hard shadow ray toward each light that reaches the surface
// the pixel's nearest sample shows (ADR 0020), and keeps a bit per light that
// its ray found hidden, for the lit pass to read. The bits follow the pixel's
// lights in the order the lit pass sums them: the directional lights, then
// the point lights its Froxel lists, as many as SHADOW_MASK_LIGHTS, across
// the word's two halves; the second half's top bits hold the Froxel's slice.
// A light past them casts no shadow there, and is counted. The depth target
// holds both culling phases' Instances when it runs; the surface's position
// is rebuilt from the sample's depth, and its face normal from the depths
// the same sample holds at the neighbouring pixels. Its bindings are
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

// Whether either TLAS holds geometry within `distance` of the surface point
// at `position`, relative to the camera, along `direction`, a unit vector:
// one hard shadow ray (ADR 0020), traced against the static TLAS, then,
// where that finds nothing, against the dynamic TLAS unless it is empty: a
// query of an empty TLAS still costs. It starts off the surface along
// `normal`, its unit face normal, in the frame both TLAS share, whose origin
// is the corner of the camera's `tlas_anchor`, composed as an Instance's
// position is (ADR 0014).
fn shadowed(position: vec3<f32>, normal: vec3<f32>, direction: vec3<f32>, distance: f32) -> bool {
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
    // Opaque, so one rayQueryProceed completes a traversal, and any hit
    // shadows the point.
    let ray = RayDesc(
        RAY_FLAG_FORCE_OPAQUE | RAY_FLAG_TERMINATE_ON_FIRST_HIT,
        0xffu,
        0.0,
        distance,
        tlas_position + normal * offset,
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

// The step in position from `position`, where `sample` of `pixel` shows a
// surface at `depth`, to where the same sample shows one at the pixel
// `axis` away, or at the pixel `axis` back, whichever is nearer in depth and
// so likelier on the same surface, taken in the direction of `axis`; zero
// where neither shows a surface.
fn neighbour_step(
    pixel: vec2<i32>,
    axis: vec2<i32>,
    sample: u32,
    depth: f32,
    position: vec3<f32>,
    size: vec2<i32>
) -> vec3<f32> {
    let ahead = pixel + axis;
    let behind = pixel - axis;
    let ahead_inside = all(ahead >= vec2<i32>(0)) && all(ahead < size);
    let behind_inside = all(behind >= vec2<i32>(0)) && all(behind < size);
    let ahead_depth = select(0.0, textureLoad(depth_target, clamp(ahead, vec2<i32>(0), size - 1), sample), ahead_inside);
    let behind_depth = select(0.0, textureLoad(depth_target, clamp(behind, vec2<i32>(0), size - 1), sample), behind_inside);
    if ahead_depth == 0.0 && behind_depth == 0.0 {
        return vec3<f32>(0.0);
    }
    let take_ahead = behind_depth == 0.0
        || (ahead_depth != 0.0 && abs(ahead_depth - depth) <= abs(behind_depth - depth));
    let neighbour = select(behind, ahead, take_ahead);
    let neighbour_position = position_at(
        camera.view_proj,
        vec2<f32>(neighbour) + sample_offset(sample),
        select(behind_depth, ahead_depth, take_ahead),
        vec2<f32>(size)
    );
    return select(position - neighbour_position, neighbour_position - position, take_ahead);
}

// The unit face normal of the surface that `sample` of `pixel` shows at
// `depth` and `position`, facing the camera, from the steps to its
// neighbours across and down; toward the camera where it has none along an
// axis.
fn face_normal(pixel: vec2<i32>, sample: u32, depth: f32, position: vec3<f32>, size: vec2<i32>) -> vec3<f32> {
    let across = neighbour_step(pixel, vec2<i32>(1, 0), sample, depth, position, size);
    let down = neighbour_step(pixel, vec2<i32>(0, 1), sample, depth, position, size);
    let normal = cross(across, down);
    let length_squared = dot(normal, normal);
    let toward_camera = -normalize(position);
    if length_squared == 0.0 {
        return toward_camera;
    }
    let unit = normal * inverseSqrt(length_squared);
    return select(unit, -unit, dot(unit, toward_camera) < 0.0);
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
        let normal = face_normal(pixel, sample, depth, position, size);
        var overflowed = 0u;
        for (var index = 0u; index < light.directional_light_count; index++) {
            if index >= SHADOW_MASK_LIGHTS {
                overflowed++;
                continue;
            }
            // A light the face turns from is hidden by the surface itself,
            // with no ray.
            let direction = normalize(-lights[index].offset);
            if dot(normal, direction) <= 0.0
                || shadowed(position, normal, direction, UNBOUNDED_RAY_DISTANCE) {
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
            if dot(normal, to_light) <= 0.0
                || shadowed(position, normal, to_light / distance, distance) {
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

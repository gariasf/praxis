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

// A shadow ray's origin leaves its surface along the normal by the larger of
// two distances. The first is this share of the point's largest coordinate,
// relative to the camera or in the TLAS's frame, as a ray's hit point is
// offset in proportion to its distance from the origin
// [ref:wachter-binder-2019-offset-error], and of 1 m at least: rebuilding
// the point from its depth, composing it into the TLAS's frame and
// intersecting the TLAS each round a coordinate by a few times 2^-24 of its
// size at most, well inside the offset's 256.
const SHADOW_RAY_OFFSET_SCALE: f32 = 1.52587890625e-5; // 2^-16

// The second is this share of a pixel's width at the point. The depth the
// point is rebuilt from lies on the triangle as rasterised, from vertices
// snapped to a grid of 1/256 of a pixel on D3D11
// [ref:d3d11.3-rasterizer-precision] and as coarse as 1/16 under Vulkan
// [ref:vulkan-subpixel-precision-bits], so the rebuilt point can lie off the
// TLAS's triangle by a share of a pixel's width at it: about half a step of
// that grid, by estimate, as no specification bounds it. Measured on the
// M4 Pro, whose grid Metal does not document, every Reference scene's
// points lie within 0.0031 of a pixel's width of their triangles, from
// 320x180 to 2560x1440, while the first distance spans as little as 0.0033
// of a pixel's width at 320x180 through the camera's 45° field of view, and
// less through a wider one. This share is ten times the measured error and
// half a step of a 1/16 grid.
const SHADOW_RAY_PIXEL_OFFSET: f32 = 0.03125; // 2^-5

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

// The cosines, of a light's angle from a plane, past which a sample's normals
// face the light or turn from it: sin 2° for the first two, the stored
// normals' error, and sin 10° for the third. The surface normals'
// 8-bit octahedral coordinates hold each within 0.95° where the store rounds
// to the nearest step, as Metal's does, and about twice that where it may
// round to either (`octahedral_decode`). Within the first, the stored
// interpolated normal may turn from a light the lit pass's still faces; a
// triangle faces a light its ray then clears by the second where its
// normals agree, as the ray, starting above the triangle's plane, cannot
// meet it again; and by the third where the interpolated normal bends from
// it; short of them, its ray starts inside the surface.
const INTERPOLATED_TURN_MARGIN: f32 = 0.03489949670250097;
const AGREEING_TRIANGLE_TURN_MARGIN: f32 = 0.03489949670250097;
const TRIANGLE_TURN_MARGIN: f32 = 0.17364817766693033;

// The cosine within which a triangle's normal and the normal interpolated
// over it agree, as on a flat surface or a flat-shaded mesh: 3°.
const NORMALS_AGREE: f32 = 0.9986295347545738;

// The first triangle either TLAS holds from `start_distance` to
// `end_distance` (the ray's tmin and tmax) along the ray from `origin`, in
// the TLAS's frame, along `direction`, a unit vector: with
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
fn first_intersection(flags: u32, origin: vec3<f32>, direction: vec3<f32>, start_distance: f32, end_distance: f32) -> RayIntersection {
    // Opaque, so one rayQueryProceed completes a traversal.
    var query: ray_query;
    rayQueryInitialize(&query, static_tlas, RayDesc(RAY_FLAG_FORCE_OPAQUE | flags, 0xffu, start_distance, end_distance, origin, direction));
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
    rayQueryInitialize(&query, dynamic_tlas, RayDesc(RAY_FLAG_FORCE_OPAQUE | flags, 0xffu, start_distance, dynamic_end_distance, origin, direction));
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
// shadows the point. `pixel_width` is a pixel's width at `position`.
fn shadowed(
    position: vec3<f32>,
    pixel_width: f32,
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
    let offset = max(
        SHADOW_RAY_OFFSET_SCALE * max(max(extent.x, extent.y), max(extent.z, 1.0)),
        SHADOW_RAY_PIXEL_OFFSET * pixel_width
    );
    let facing = dot(triangle_normal, direction);
    let leaves_surface = facing >= TRIANGLE_TURN_MARGIN
        || (facing >= AGREEING_TRIANGLE_TURN_MARGIN
            && dot(triangle_normal, interpolated_normal) >= NORMALS_AGREE);
    let origin = tlas_position + triangle_normal * select(-offset, offset, leaves_surface);
    // Any hit shadows a ray that leaves the surface. One that starts inside
    // first finds where it leaves the mesh, then traces on from there: two
    // turns at most, through the one call.
    var flags = select(0u, RAY_FLAG_TERMINATE_ON_FIRST_HIT, leaves_surface);
    var start_distance = 0.0;
    var hidden = false;
    loop {
        let hit = first_intersection(flags, origin, direction, start_distance, distance);
        let found = hit.kind != RAY_QUERY_INTERSECTION_NONE;
        let any_hit = (flags & RAY_FLAG_TERMINATE_ON_FIRST_HIT) != 0u;
        if any_hit || !found || hit.front_face || hit.t + offset >= distance {
            hidden = found && (any_hit || hit.front_face);
            break;
        }
        flags = RAY_FLAG_TERMINATE_ON_FIRST_HIT;
        start_distance = hit.t + offset;
    }
    return hidden;
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

// The surface that `sample` of `pixel` shows, at `depth`: where it lies,
// relative to the camera, a pixel's width there, its triangle's normal and
// the normal interpolated over it.
struct SampleSurface {
    position: vec3<f32>,
    pixel_width: f32,
    triangle_normal: vec3<f32>,
    interpolated_normal: vec3<f32>,
}

fn sample_surface(pixel: vec2<i32>, sample: u32, depth: f32, size: vec2<f32>) -> SampleSurface {
    let point = vec2<f32>(pixel) + sample_offset(sample);
    let position = position_at(camera.view_proj, point, depth, size);
    let normals = textureLoad(surface_normals, pixel, sample) * 2.0 - 1.0;
    return SampleSurface(
        position,
        distance(position, position_at(camera.view_proj, point + vec2<f32>(1.0, 0.0), depth, size)),
        octahedral_decode(normals.xy),
        octahedral_decode(normals.zw)
    );
}

// Whether the directional light at `index` of the light buffer is hidden
// from `surface`: a light the interpolated normal turns from is hidden by
// the surface itself, with no ray.
fn directional_hidden(surface: SampleSurface, index: u32) -> bool {
    let direction = normalize(-lights[index].offset);
    return !(dot(surface.interpolated_normal, direction) > -INTERPOLATED_TURN_MARGIN)
        || shadowed(surface.position, surface.pixel_width, surface.triangle_normal, surface.interpolated_normal, direction, UNBOUNDED_RAY_DISTANCE);
}

// The bits for the pixel's lights whose rays a surface found hidden, with
// the slice of the Froxel they follow in the second half's top bits, and
// which of the first SHADOW_EDGE_LIGHTS directional lights it traced a ray
// toward.
struct SurfaceWord {
    word: vec2<u32>,
    traced_directional_lights: u32,
}

// The word of the surface that the pixel's `sample` shows, at `depth`;
// none where `depth` is the background's.
fn surface_word(pixel: vec2<i32>, sample: u32, depth: f32, size: vec2<f32>) -> SurfaceWord {
    var hidden = vec2<u32>(0u);
    var traced = 0u;
    var slice = 0u;
    // Depth 0 is the far plane, the background, which nothing shadows.
    if depth > 0.0 {
        let surface = sample_surface(pixel, sample, depth, size);
        let position = surface.position;
        let triangle_normal = surface.triangle_normal;
        let interpolated_normal = surface.interpolated_normal;
        var overflowed = 0u;
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
        // The directional lights, then the point lights, in one loop with
        // one call that traces, so the shader holds one ray query's state
        // (`first_intersection`). Each light's bit is its entry.
        let light_count = light.directional_light_count + point_light_count;
        for (var entry = 0u; entry < light_count; entry++) {
            var direction: vec3<f32>;
            var distance: f32;
            if entry < light.directional_light_count {
                if entry >= SHADOW_MASK_LIGHTS {
                    overflowed++;
                    continue;
                }
                // A light the interpolated normal turns from is hidden by the
                // surface itself, with no ray.
                direction = normalize(-lights[entry].offset);
                distance = UNBOUNDED_RAY_DISTANCE;
                if !(dot(interpolated_normal, direction) > -INTERPOLATED_TURN_MARGIN) {
                    hidden = with_bit(hidden, entry);
                    continue;
                }
                if entry < SHADOW_EDGE_LIGHTS {
                    traced |= 1u << entry;
                }
            } else {
                var index = entry;
                if FROXEL_LIGHTS {
                    index = froxel_lists[froxel].lights[entry - light.directional_light_count];
                }
                let point = lights[index];
                let to_light = point_light_position(point, camera.anchor, camera.offset) - position;
                let distance_squared = dot(to_light, to_light);
                if distance_squared >= point.range * point.range {
                    continue;
                }
                if entry >= SHADOW_MASK_LIGHTS {
                    overflowed++;
                    continue;
                }
                // Facing first, so a light at the surface, at no distance, is
                // hidden without dividing by it.
                distance = sqrt(distance_squared);
                if dot(interpolated_normal, to_light) <= -INTERPOLATED_TURN_MARGIN * distance {
                    hidden = with_bit(hidden, entry);
                    continue;
                }
                direction = to_light / distance;
            }
            if shadowed(position, surface.pixel_width, triangle_normal, interpolated_normal, direction, distance) {
                hidden = with_bit(hidden, entry);
            }
        }
        if overflowed > 0u {
            atomicAdd(&shadow_mask_lights_overflowed, overflowed);
        }
    }
    let slice_shift = SHADOW_MASK_LIGHTS - 32u;
    return SurfaceWord(vec2<u32>(hidden.x, hidden.y | (slice << slice_shift)), traced);
}

// The index of the first of four samples whose value in `values` is `value`.
fn first_sample_of(values: vec4<f32>, value: f32) -> u32 {
    return select(
        select(select(3u, 2u, values.z == value), 1u, values.y == value),
        0u,
        values.x == value
    );
}

// Pixels listed for a later dispatch to trace: how many, then each one's
// pixel, its column in the low 16 bits and its row above them, and, for a
// second surface, from bit 30 the sample to trace it from.
struct PixelList {
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
        let surface = sample_surface(pixel, sample, depth, target_size);
        let position = surface.position;
        let plane_normal = surface.triangle_normal;
        let off_plane = vec4<f32>(
            dot(position_at(camera.view_proj, vec2<f32>(pixel) + SAMPLE_OFFSETS[0], depths.x, target_size) - position, plane_normal),
            dot(position_at(camera.view_proj, vec2<f32>(pixel) + SAMPLE_OFFSETS[1], depths.y, target_size) - position, plane_normal),
            dot(position_at(camera.view_proj, vec2<f32>(pixel) + SAMPLE_OFFSETS[2], depths.z, target_size) - position, plane_normal),
            dot(position_at(camera.view_proj, vec2<f32>(pixel) + SAMPLE_OFFSETS[3], depths.w, target_size) - position, plane_normal)
        );
        let second = abs(off_plane) > vec4<f32>(surface.pixel_width) & depths > vec4<f32>(0.0);
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
    let nearest = surface_word(pixel, sample, depth, target_size);
    textureStore(shadow_mask, pixel, 0, vec4<u32>(nearest.word, 0u, 0u));
    // The first SHADOW_EDGE_LIGHTS directional lights' bits, each the
    // verdict at every sample of the nearest surface until
    // `trace_shadow_edges` traces it again from the others.
    let edge_light_count = min(light.directional_light_count, SHADOW_EDGE_LIGHTS);
    let hidden = nearest.word.x & ((1u << edge_light_count) - 1u);
    let sample_verdicts = at_every_sample(hidden, edge_light_count);
    textureStore(
        shadow_mask_samples,
        pixel,
        vec4<u32>(second_samples | (sample_verdicts << SAMPLE_COUNT), 0u, 0u, 0u)
    );
    directional_verdicts[invocation.y * u32(size.x) + invocation.x] = second_samples
        | (hidden << VERDICT_HIDDEN_SHIFT)
        | (nearest.traced_directional_lights << VERDICT_TRACED_SHIFT);
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
    let surface = surface_word(pixel, sample, depth, vec2<f32>(textureDimensions(depth_target)));
    textureStore(shadow_mask, pixel, 1, vec4<u32>(surface.word, 0u, 0u));
}

// The first `light_count` lights' bits of `hidden`, each repeated at every
// sample, SAMPLE_COUNT bits per light, as the lit pass reads them from the
// shadow mask samples above the second surface's samples.
fn at_every_sample(hidden: u32, light_count: u32) -> u32 {
    var sample_verdicts = 0u;
    for (var index = 0u; index < light_count; index++) {
        sample_verdicts |= ((hidden >> index) & 1u) * (ALL_SAMPLES << (SAMPLE_COUNT * index));
    }
    return sample_verdicts;
}

// Where a pixel's word in `directional_verdicts` holds, for its nearest
// surface and the first SHADOW_EDGE_LIGHTS directional lights, a bit per
// light its ray or its turn found hidden, and a bit per light it traced a
// ray toward, above the samples that show its second surface.
const VERDICT_HIDDEN_SHIFT: u32 = 8u;
const VERDICT_TRACED_SHIFT: u32 = 16u;

// The first SHADOW_EDGE_LIGHTS directional lights, a bit each.
const EDGE_LIGHTS: u32 = (1u << SHADOW_EDGE_LIGHTS) - 1u;

// The lights of `own`'s verdicts that differ from `neighbour`'s, among those
// both traced a ray toward: a surface turned from a light is hidden with no
// ray, and its shading already fades the light out at its terminator.
fn differing_lights(own: u32, neighbour: u32) -> u32 {
    return ((own ^ neighbour) >> VERDICT_HIDDEN_SHIFT)
        & (own >> VERDICT_TRACED_SHIFT)
        & (neighbour >> VERDICT_TRACED_SHIFT)
        & EDGE_LIGHTS;
}

// The directional lights whose verdicts at the nearest surface of `pixel`,
// whose word is `own`, differ from a 4-neighbour's, a target's side standing
// in for the neighbour past it. A directional light's bit is the same at
// every pixel, whatever Froxel's point lights follow it.
fn shadow_edge_lights(pixel: vec2<i32>, own: u32, size: vec2<i32>) -> u32 {
    if (own >> VERDICT_TRACED_SHIFT) == 0u {
        return 0u;
    }
    let last = size - 1;
    let width = size.x;
    let left = directional_verdicts[pixel.y * width + max(pixel.x - 1, 0)];
    let right = directional_verdicts[pixel.y * width + min(pixel.x + 1, last.x)];
    let up = directional_verdicts[max(pixel.y - 1, 0) * width + pixel.x];
    let down = directional_verdicts[min(pixel.y + 1, last.y) * width + pixel.x];
    return differing_lights(own, left) | differing_lights(own, right)
        | differing_lights(own, up) | differing_lights(own, down);
}

// The directional lights that the nearest surface of the pixel whose word is
// `own` turns from, hidden with no ray (`surface_word`).
fn turned_lights(own: u32) -> u32 {
    return (own >> VERDICT_HIDDEN_SHIFT) & ~(own >> VERDICT_TRACED_SHIFT) & EDGE_LIGHTS;
}

// The lights the nearest sample of `pixel`, whose word is `own`, turns from
// that another sample of its nearest surface faces by its own interpolated
// normal. At a silhouette the nearest triangle is seen nearly edge-on, and
// the surface behind it can lie within a pixel's width of its plane, so the
// pixel shows one surface by depth while its other samples face a light the
// nearest sample turns from.
fn turned_lights_other_samples_face(pixel: vec2<i32>, own: u32) -> u32 {
    let turned = turned_lights(own);
    if turned == 0u {
        return 0u;
    }
    let depths = vec4<f32>(
        textureLoad(depth_target, pixel, 0),
        textureLoad(depth_target, pixel, 1),
        textureLoad(depth_target, pixel, 2),
        textureLoad(depth_target, pixel, 3)
    );
    let nearest = first_sample_of(depths, max(max(depths.x, depths.y), max(depths.z, depths.w)));
    // The background's samples and the second surface's are not the nearest
    // surface's.
    let shown = select(vec4<u32>(0u), vec4<u32>(1u, 2u, 4u, 8u), depths > vec4<f32>(0.0));
    let other_nearest_samples = (shown.x | shown.y | shown.z | shown.w) & ~(own & ALL_SAMPLES) & ~(1u << nearest);
    var faced = 0u;
    for (var sample = 0u; sample < SAMPLE_COUNT; sample++) {
        if ((other_nearest_samples >> sample) & 1u) == 0u {
            continue;
        }
        let normals = textureLoad(surface_normals, pixel, sample) * 2.0 - 1.0;
        let interpolated_normal = octahedral_decode(normals.zw);
        for (var index = 0u; index < SHADOW_EDGE_LIGHTS; index++) {
            if ((turned >> index) & 1u) == 1u
                && dot(interpolated_normal, normalize(-lights[index].offset)) > -INTERPOLATED_TURN_MARGIN {
                faced |= 1u << index;
            }
        }
    }
    return faced;
}

// Lists each pixel at a directional light's shadow edge, whose verdict
// differs from a 4-neighbour's or between its samples, for
// `trace_shadow_edges`: as with second surfaces, the extra rays of a few
// pixels traced in place would hold up their SIMD groups
// [ref:lauritzen-2010-per-sample-scheduling]. It holds no ray query, so the
// samples' normals are read here rather than in `build_shadow_mask`, whose
// ray queries' registers already bound its occupancy.
@compute @workgroup_size(SHADOW_MASK_TILE_SIZE, SHADOW_MASK_TILE_SIZE)
fn list_shadow_edges(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let size = vec2<i32>(textureDimensions(depth_target));
    let pixel = vec2<i32>(invocation.xy);
    if any(pixel >= size) {
        return;
    }
    let own = directional_verdicts[pixel.y * size.x + pixel.x];
    if shadow_edge_lights(pixel, own, size) == 0u && turned_lights_other_samples_face(pixel, own) == 0u {
        return;
    }
    let entry = atomicAdd(&shadow_edges.count, 1u);
    shadow_edges.entries[entry] = invocation.x | (invocation.y << 16u);
    let group_size = SHADOW_MASK_TILE_SIZE * SHADOW_MASK_TILE_SIZE;
    if entry % group_size == 0u {
        let groups = entry / group_size + 1u;
        atomicMax(&shadow_edge_dispatch[0], min(groups, MAX_WORKGROUPS_PER_DIMENSION));
        atomicMax(
            &shadow_edge_dispatch[1],
            (groups + MAX_WORKGROUPS_PER_DIMENSION - 1u) / MAX_WORKGROUPS_PER_DIMENSION
        );
        atomicStore(&shadow_edge_dispatch[2], 1u);
    }
}

// Traces each pixel `list_shadow_edges` listed again, toward each
// directional light whose verdict differs from a neighbour's or that its
// nearest sample turns from, from each other sample that shows its nearest
// surface, and keeps each sample's verdict for the lit pass, which shades a
// light by the share of its fragment's samples that see it. A sample that
// turns from a light too is hidden from it with no ray.
@compute @workgroup_size(SHADOW_MASK_TILE_SIZE * SHADOW_MASK_TILE_SIZE)
fn trace_shadow_edges(
    @builtin(workgroup_id) group: vec3<u32>,
    @builtin(num_workgroups) groups: vec3<u32>,
    @builtin(local_invocation_index) lane: u32
) {
    if lane == 0u {
        listed_count = atomicLoad(&shadow_edges.count);
    }
    let entry = (group.y * groups.x + group.x) * SHADOW_MASK_TILE_SIZE * SHADOW_MASK_TILE_SIZE + lane;
    if entry >= workgroupUniformLoad(&listed_count) {
        return;
    }
    let listed = shadow_edges.entries[entry];
    let pixel = vec2<i32>(vec2<u32>(listed & 0xffffu, listed >> 16u));
    let size = vec2<i32>(textureDimensions(depth_target));
    let own = directional_verdicts[pixel.y * size.x + pixel.x];
    let edge_lights = shadow_edge_lights(pixel, own, size) | turned_lights(own);
    let second_samples = own & ALL_SAMPLES;
    let depths = vec4<f32>(
        textureLoad(depth_target, pixel, 0),
        textureLoad(depth_target, pixel, 1),
        textureLoad(depth_target, pixel, 2),
        textureLoad(depth_target, pixel, 3)
    );
    let nearest = first_sample_of(depths, max(max(depths.x, depths.y), max(depths.z, depths.w)));
    // Each light's verdict at the nearest sample, at every sample, until
    // traced again.
    let edge_light_count = min(light.directional_light_count, SHADOW_EDGE_LIGHTS);
    var sample_verdicts = at_every_sample((own >> VERDICT_HIDDEN_SHIFT) & EDGE_LIGHTS, edge_light_count);
    for (var sample = 0u; sample < SAMPLE_COUNT; sample++) {
        let depth = depths[sample];
        // The nearest sample's verdict stands; another surface's samples
        // and the background's keep theirs.
        if sample == nearest || ((second_samples >> sample) & 1u) == 1u || depth <= 0.0 {
            continue;
        }
        let surface = sample_surface(pixel, sample, depth, vec2<f32>(size));
        for (var index = 0u; index < edge_light_count; index++) {
            if ((edge_lights >> index) & 1u) == 0u {
                continue;
            }
            let bit = 1u << (SAMPLE_COUNT * index + sample);
            sample_verdicts = select(sample_verdicts & ~bit, sample_verdicts | bit, directional_hidden(surface, index));
        }
    }
    textureStore(
        shadow_mask_samples,
        pixel,
        vec4<u32>(second_samples | (sample_verdicts << SAMPLE_COUNT), 0u, 0u, 0u)
    );
}

// The Froxel pass (src/render/froxel_pass.rs): a workgroup per screen
// rectangle and an invocation per slice, each listing the point lights whose
// range reaches into its Froxel in light order, so the lit pass sums a
// pixel's lights in the order brute force does, and counting those its list
// has no room for.

@compute @workgroup_size(FROXEL_SLICES)
fn build_froxel_lists(
    @builtin(workgroup_id) rectangle: vec3<u32>,
    @builtin(local_invocation_index) slice: u32
) {
    // The rectangle's sides in normalised device coordinates, its lowest
    // corner and its highest, and its slice's view depths; the last slice
    // reaches to infinity.
    let lowest = vec2<f32>(rectangle.xy) / FROXEL_GRID_SIZE * 2.0 - 1.0;
    let highest = vec2<f32>(rectangle.xy + 1u) / FROXEL_GRID_SIZE * 2.0 - 1.0;
    let nearest = froxel_slice_start(slice);
    let farthest = froxel_slice_start(slice + 1u);
    let last_slice = slice == FROXEL_SLICES - 1u;
    // Each side is a plane through the camera, since view_proj holds no
    // translation (ADR 0014): a point is inside the lowest x side where
    // clip x − lowest.x clip w ≥ 0, and that difference over the length of
    // the plane's normal, row x − lowest.x row w of view_proj, is its
    // distance from the plane. Clip w is the depth along the view axis.
    let rows = transpose(camera.view_proj);
    let normal_lengths = vec4<f32>(
        length(rows[0].xyz - lowest.x * rows[3].xyz),
        length(highest.x * rows[3].xyz - rows[0].xyz),
        length(rows[1].xyz - lowest.y * rows[3].xyz),
        length(highest.y * rows[3].xyz - rows[1].xyz)
    );

    let froxel = froxel_index(rectangle.xy, slice);
    var count = 0u;
    let end = light.directional_light_count + light.point_light_count;
    for (var index = light.directional_light_count; index < end; index++) {
        let point = lights[index];
        let centre = point_light_position(point, camera.anchor, camera.offset);
        let clip_x = dot(rows[0].xyz, centre);
        let clip_y = dot(rows[1].xyz, centre);
        let depth = dot(rows[3].xyz, centre);
        let inside_sides = vec4<f32>(
            clip_x - lowest.x * depth,
            highest.x * depth - clip_x,
            clip_y - lowest.y * depth,
            highest.y * depth - clip_y
        );
        // Its range reaches into the Froxel unless it lies wholly outside a
        // side or beyond the slice: conservative, as a sphere may miss a
        // corner that it reaches past both sides of.
        if all(inside_sides >= -point.range * normal_lengths)
            && depth + point.range >= nearest
            && (last_slice || depth - point.range <= farthest) {
            if count < FROXEL_LIGHT_CAPACITY {
                froxel_lists[froxel].lights[count] = index;
            }
            count++;
        }
    }
    froxel_lists[froxel].light_count = min(count, FROXEL_LIGHT_CAPACITY);
    if count > FROXEL_LIGHT_CAPACITY {
        atomicAdd(&froxel_lights_overflowed, count - FROXEL_LIGHT_CAPACITY);
    }
}

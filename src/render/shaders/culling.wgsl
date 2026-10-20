// The culling pass's entry points, after the shared parts: they append the
// Meshlets of the Instances they draw to the frame's draw in slot order,
// which the main pass draws with one indirect call (ADR 0019). With the
// CULLING Render switch on, they draw only the Instances whose Mesh bounds
// reach into the view frustum. Its bindings are declared from their bind
// groups' definitions (src/render/bind_groups.rs).
//
// Slot order keeps the draw the same from run to run: on the M4 Pro, which
// triangles a draw holds before another changes the derivatives some pixels
// sample their Textures with.

// The frame's draw: the indirect arguments of one indexed draw from index 0,
// cleared before the pass.
struct FrameDraw {
    index_count: atomic<u32>,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
}

var<workgroup> scan: array<vec2<u32>, SCAN_BLOCK_SIZE>;

// The offsets of a slot the pass does not draw, freed or culled: past every
// capacity, as a sum past a u32 saturates to.
const NOT_DRAWN = vec2<u32>(0xffffffffu);

// `sum + addend`, or the most a u32 holds where that would pass it: past
// every capacity, as the sum it stands for is.
fn saturating_add(sum: vec2<u32>, addend: vec2<u32>) -> vec2<u32> {
    return select(sum + addend, vec2<u32>(0xffffffffu), sum > vec2<u32>(0xffffffffu) - addend);
}

// The sum of `value` over the workgroup's invocations before this one, which
// they all call together: a Hillis and Steele scan, read one place back.
fn exclusive_scan(invocation: u32, value: vec2<u32>) -> vec2<u32> {
    scan[invocation] = value;
    for (var stride = 1u; stride < SCAN_BLOCK_SIZE; stride *= 2u) {
        workgroupBarrier();
        var before = vec2<u32>(0u);
        if invocation >= stride {
            before = scan[invocation - stride];
        }
        workgroupBarrier();
        scan[invocation] = saturating_add(scan[invocation], before);
    }
    workgroupBarrier();
    var before = vec2<u32>(0u);
    if invocation > 0u {
        before = scan[invocation - 1u];
    }
    return before;
}

// How far inside each side plane of the view frustum, x ≥ −w, x ≤ w,
// y ≥ −w and y ≤ w, a clip-space position lies, or a clip-space direction
// moves.
fn side_plane_distances(clip: vec4<f32>) -> vec4<f32> {
    return clip.w + vec4<f32>(clip.x, -clip.x, clip.y, -clip.y);
}

// How far inside the view frustum's near plane, z ≤ w, a clip-space
// position lies, or a clip-space direction moves.
fn near_plane_distance(clip: vec4<f32>) -> f32 {
    return clip.w - clip.z;
}

// Whether an Instance's Mesh bounds reach into the view frustum, which clip
// space bounds by its four side planes and its near plane; with reverse-Z
// and no far plane, every point past the near plane lands at z ≥ 0, so
// there is no sixth plane (ADR 0015). The box, its centre composed relative
// to the camera as its vertices are (ADR 0014), is outside only where it
// lies wholly beyond one plane. A plane's distance is linear in clip space,
// so the box reaches furthest inside it by its centre's distance plus each
// half axis's, whole.
fn in_view_frustum(instance: InstanceData, mesh: MeshData) -> bool {
    let bounds_min = vec3<f32>(mesh.bounds_min[0], mesh.bounds_min[1], mesh.bounds_min[2]);
    let bounds_max = vec3<f32>(mesh.bounds_max[0], mesh.bounds_max[1], mesh.bounds_max[2]);
    let half_extent = 0.5 * (bounds_max - bounds_min);
    let centre = camera_relative_position(
        instance.anchor,
        instance.offset,
        camera.anchor,
        camera.offset,
        instance.rotation_scale * (0.5 * (bounds_min + bounds_max))
    );
    let clip_centre = camera.view_proj * vec4<f32>(centre, 1.0);
    let clip_half_axes = camera.view_proj * mat3x4<f32>(
        vec4<f32>(instance.rotation_scale[0] * half_extent.x, 0.0),
        vec4<f32>(instance.rotation_scale[1] * half_extent.y, 0.0),
        vec4<f32>(instance.rotation_scale[2] * half_extent.z, 0.0),
    );
    let side_reach = side_plane_distances(clip_centre)
        + abs(side_plane_distances(clip_half_axes[0]))
        + abs(side_plane_distances(clip_half_axes[1]))
        + abs(side_plane_distances(clip_half_axes[2]));
    let near_reach = near_plane_distance(clip_centre)
        + abs(near_plane_distance(clip_half_axes[0]))
        + abs(near_plane_distance(clip_half_axes[1]))
        + abs(near_plane_distance(clip_half_axes[2]));
    return all(side_reach >= vec4<f32>(0.0)) && near_reach >= 0.0;
}

// A workgroup per block of SCAN_BLOCK_SIZE slots: each slot's Meshlet
// records and packed indices, its Instance's where the pass draws it or none,
// summed over the block's earlier slots into its offsets, NOT_DRAWN in their
// place where it is not drawn, and the block's sum after every slot's.
@compute @workgroup_size(SCAN_BLOCK_SIZE)
fn offset_slots(
    @builtin(workgroup_id) block: vec3<u32>,
    @builtin(local_invocation_index) invocation: u32
) {
    let slot = block.x * SCAN_BLOCK_SIZE + invocation;
    var drawn = false;
    var counts = vec2<u32>(0u);
    if slot < instance_slot_count && instances[slot].mesh != NO_MESH {
        let instance = instances[slot];
        let mesh = meshes[instance.mesh];
        drawn = !CULLING || in_view_frustum(instance, mesh);
        if drawn {
            counts = vec2<u32>(mesh.meshlet_count, 3u * mesh.triangle_count);
        }
    }
    let offset = exclusive_scan(invocation, counts);
    if slot < instance_slot_count {
        instance_offsets[slot] = select(NOT_DRAWN, offset, drawn);
    }
    if invocation == SCAN_BLOCK_SIZE - 1u {
        instance_offsets[INSTANCE_CAPACITY + block.x] = saturating_add(offset, counts);
    }
}

// One workgroup: each block's offsets, the sum of the earlier blocks', in
// place of its own sum; and the draw is of one Instance. Each invocation
// takes a run of blocks, its loops bounded alike in every invocation so the
// scan's barriers stay in uniform control flow.
@compute @workgroup_size(SCAN_BLOCK_SIZE)
fn offset_blocks(@builtin(local_invocation_index) invocation: u32) {
    let block_count = (instance_slot_count + SCAN_BLOCK_SIZE - 1u) / SCAN_BLOCK_SIZE;
    let blocks_per_invocation = (block_count + SCAN_BLOCK_SIZE - 1u) / SCAN_BLOCK_SIZE;
    let first_block = invocation * blocks_per_invocation;
    var sum = vec2<u32>(0u);
    for (var step = 0u; step < blocks_per_invocation; step++) {
        if first_block + step < block_count {
            sum = saturating_add(sum, instance_offsets[INSTANCE_CAPACITY + first_block + step]);
        }
    }
    var offset = exclusive_scan(invocation, sum);
    for (var step = 0u; step < blocks_per_invocation; step++) {
        let block = INSTANCE_CAPACITY + first_block + step;
        if first_block + step < block_count {
            let block_sum = instance_offsets[block];
            instance_offsets[block] = offset;
            offset = saturating_add(offset, block_sum);
        }
    }
    if invocation == 0u {
        frame_draw.instance_count = 1u;
    }
}

// A workgroup per Instance slot, dispatched in rows of at most 65,535, and
// an invocation per triangle of a Meshlet: writes the Instance's Meshlet
// records and packed indices from its offsets, unless they are NOT_DRAWN.
// Where they would pass either buffer's end, neither it nor any Instance
// after it is drawn, so the draw ends at the last Instance that fits, and
// every index before is written.
// ponytail: a workgroup per Instance idles most invocations on small Meshes;
// split it into a pass per Instance and one per Meshlet when a profile of a
// dense scene names it.
@compute @workgroup_size(MESHLET_MAX_TRIANGLES)
fn append_meshlets(
    @builtin(workgroup_id) workgroup: vec3<u32>,
    @builtin(num_workgroups) workgroups: vec3<u32>,
    @builtin(local_invocation_index) triangle: u32
) {
    let slot = workgroup.y * workgroups.x + workgroup.x;
    if slot >= instance_slot_count || all(instance_offsets[slot] == NOT_DRAWN) {
        return;
    }
    let instance = instances[slot];
    let mesh = meshes[instance.mesh];
    let block_offset = instance_offsets[INSTANCE_CAPACITY + slot / SCAN_BLOCK_SIZE];
    let first = saturating_add(block_offset, instance_offsets[slot]);
    let counts = vec2<u32>(mesh.meshlet_count, 3u * mesh.triangle_count);
    let capacity = vec2<u32>(arrayLength(&meshlet_records), arrayLength(&packed_indices));
    if any(first > capacity) || any(counts > capacity - first) {
        return;
    }
    if triangle == 0u {
        atomicMax(&frame_draw.index_count, first.y + counts.y);
    }

    for (var meshlet_in_mesh = 0u; meshlet_in_mesh < mesh.meshlet_count; meshlet_in_mesh++) {
        let meshlet_index = mesh.first_meshlet + meshlet_in_mesh;
        let meshlet = meshlets[meshlet_index];
        let record = first.x + meshlet_in_mesh;
        if triangle == 0u {
            let overridden = instance.material_id != NO_MATERIAL;
            let material_id = select(meshlet.material_id, instance.material_id, overridden);
            meshlet_records[record] = MeshletRecord(slot, meshlet_index, material_id);
        }
        if triangle < meshlet.triangle_count {
            // The Mesh's triangles in cooked order, each with its corners as
            // cooked, so the draw rasterizes as the cooked index stream does.
            let triangle_in_mesh = meshlet.first_triangle - mesh.first_triangle + triangle;
            let first_index = first.y + 3u * triangle_in_mesh;
            let corners = meshlet_triangles[meshlet.first_triangle + triangle];
            for (var corner = 0u; corner < 3u; corner++) {
                let vertex = extractBits(corners, 8u * corner, 8u);
                packed_indices[first_index + corner] = pack_index(record, vertex);
            }
        }
    }
}

// The culling pass's entry points, after the shared parts: they append the
// Meshlets of the Instances they draw to the frame's draw in slot order,
// which the main pass draws with one indirect call per phase and depth
// pipeline, and one of both phases for the lit pipeline (ADR 0019). Phase
// one draws the Instances visible the frame before; phase two, after the
// depth pyramid, the other Instances in view that the pyramid does not
// hide, and it records which Instances were visible for the next frame. With
// the CULLING
// Render switch on, they draw only the Instances whose Mesh bounds reach
// into the view frustum; off, phase one draws every Instance in use. Its
// bindings are declared from their bind groups' definitions
// (src/render/bind_groups.rs).
//
// Slot order keeps the draw the same from run to run: on the M4 Pro, which
// triangles a draw holds before another changes the derivatives some pixels
// sample their Textures with.

// Which phase a pipeline runs, set as it is created.
override PHASE_TWO: bool;

// The indirect arguments of one indexed draw.
struct DrawIndexedIndirect {
    index_count: atomic<u32>,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
}

// The arguments of one indirect dispatch: its workgroups in x, y and z.
struct DispatchIndirect {
    x: u32,
    y: u32,
    z: u32,
}

// The frame's draw, cleared before phase one: the draw of the Instances
// appended so far, from index 0, so phase one's, which the depth pre-pass
// draws, then both phases', which the lit pass draws; the Instances it holds
// and those it has no room for, which Frame stats read back; phase two's own
// draw, of the indices after phase one's, which the lit pass draws into
// depth before it shades; where phase two's Meshlet records and packed
// indices start, phase one's sums; the phase's dispatch of append_triangles,
// which the pass copies out to dispatch it; and where the phase's Meshlet
// records end, short of the buffer's end.
struct FrameDraw {
    draw: DrawIndexedIndirect,
    instances_drawn: atomic<u32>,
    instances_overflowed: atomic<u32>,
    phase_two_draw: DrawIndexedIndirect,
    phase_two_offsets: vec2<u32>,
    append_dispatch: DispatchIndirect,
    records_end: u32,
}

var<workgroup> scan: array<vec2<u32>, SCAN_BLOCK_SIZE>;

// Phase two's visibility bits for its block of slots, gathered before they
// are written.
var<workgroup> visible_words: array<atomic<u32>, SCAN_BLOCK_SIZE / 32u>;

// The offsets of a slot the pass does not draw, freed or culled: past every
// capacity, as a sum past a u32 saturates to.
const NOT_DRAWN = vec2<u32>(0xffffffffu);

// `sum + addend`, or the most a u32 holds where that would pass it: past
// every capacity, as the sum it stands for is.
fn saturating_add(sum: vec2<u32>, addend: vec2<u32>) -> vec2<u32> {
    return select(sum + addend, vec2<u32>(0xffffffffu), sum > vec2<u32>(0xffffffffu) - addend);
}

// The sum of `value` over the workgroup's invocations before this one, which
// they all call together: a Hillis and Steele scan
// [ref:hillis-steele-partial-sums], read one place back.
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

// An Instance's Mesh bounds in clip space: the box's centre, composed
// relative to the camera as its vertices are (ADR 0014), and its half axes,
// each a direction.
struct ClipBox {
    centre: vec4<f32>,
    half_axes: mat3x4<f32>,
}

fn clip_space_bounds(instance: InstanceData, mesh: MeshData) -> ClipBox {
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
    return ClipBox(
        camera.view_proj * vec4<f32>(centre, 1.0),
        camera.view_proj * mat3x4<f32>(
            vec4<f32>(instance.rotation_scale[0] * half_extent.x, 0.0),
            vec4<f32>(instance.rotation_scale[1] * half_extent.y, 0.0),
            vec4<f32>(instance.rotation_scale[2] * half_extent.z, 0.0),
        ),
    );
}

// Whether an Instance's Mesh bounds reach into the view frustum, which clip
// space bounds by its four side planes and its near plane; with reverse-Z
// and no far plane, every point past the near plane lands at z ≥ 0, so
// there is no sixth plane (ADR 0015). The box is outside only where it lies
// wholly beyond one plane. A plane's distance is linear in clip space, so
// the box reaches furthest inside it by its centre's distance plus each
// half axis's, whole.
fn in_view_frustum(bounds: ClipBox) -> bool {
    let side_reach = side_plane_distances(bounds.centre)
        + abs(side_plane_distances(bounds.half_axes[0]))
        + abs(side_plane_distances(bounds.half_axes[1]))
        + abs(side_plane_distances(bounds.half_axes[2]));
    let near_reach = near_plane_distance(bounds.centre)
        + abs(near_plane_distance(bounds.half_axes[0]))
        + abs(near_plane_distance(bounds.half_axes[1]))
        + abs(near_plane_distance(bounds.half_axes[2]));
    return all(side_reach >= vec4<f32>(0.0)) && near_reach >= 0.0;
}

// How far the occlusion test moves the box's corners towards the camera in
// clip-space z and w, metres for a perspective camera, past where f32
// rounding could put the vertices the vertex shader composes: a share of
// the terms summed, and ADR 0014's worst camera-relative error, 0.041 mm,
// for the box's composition and the vertices' each, rounded up.
const OCCLUSION_ROUNDING: f32 = 1.0 / 16384.0;
const COMPOSITION_ERROR: f32 = 1e-4;

// Whether a box that reaches into the view frustum lies wholly behind what
// phase one drew: nearer nowhere than the depth pyramid's minimum, the
// farthest depth (ADR 0015), over the texels its screen rectangle covers,
// in the level where that rectangle spans at most two texels per axis.
// Depth, z over w, is greatest at a corner of a box in front of the camera,
// as is the box's reach across the screen, so the test projects the eight
// corners, each moved towards the camera past f32 rounding, and grows the
// rectangle by a pixel past where rasterization rounds vertices. A box
// with a corner at or behind the camera's plane, or that rounding leaves
// over more than two texels, is not occluded.
fn occluded(bounds: ClipBox) -> bool {
    let axes = bounds.half_axes;
    let w_rounding = OCCLUSION_ROUNDING
        * (abs(bounds.centre.w) + abs(axes[0].w) + abs(axes[1].w) + abs(axes[2].w))
        + COMPOSITION_ERROR;
    let z_rounding = OCCLUSION_ROUNDING
        * (abs(bounds.centre.z) + abs(axes[0].z) + abs(axes[1].z) + abs(axes[2].z))
        + COMPOSITION_ERROR;
    var nearest_depth = 0.0;
    var low = vec2<f32>(1.0);
    var high = vec2<f32>(-1.0);
    for (var corner = 0u; corner < 8u; corner++) {
        let signs = 2.0 * vec3<f32>((vec3<u32>(corner) >> vec3<u32>(0u, 1u, 2u)) & vec3<u32>(1u)) - 1.0;
        let clip = bounds.centre + axes * signs;
        if clip.w <= w_rounding {
            return false;
        }
        nearest_depth = max(nearest_depth, (clip.z + z_rounding) / (clip.w - w_rounding));
        low = min(low, clip.xy / clip.w);
        high = max(high, clip.xy / clip.w);
    }
    // In texture coordinates, rows top first: a level-0 texel spans two
    // pixels, or a little more on an odd side.
    let level_zero_texels = textureDimensions(depth_pyramid, 0);
    let level_zero_size = vec2<f32>(level_zero_texels);
    let pixel_margin = 0.5 / level_zero_size;
    let first_coordinates = saturate(vec2<f32>(0.5 * low.x + 0.5, 0.5 - 0.5 * high.y) - pixel_margin);
    let last_coordinates = saturate(vec2<f32>(0.5 * high.x + 0.5, 0.5 - 0.5 * low.y) + pixel_margin);
    let extent = (last_coordinates - first_coordinates) * level_zero_size;
    let level = min(
        u32(ceil(log2(max(max(extent.x, extent.y), 1.0)))),
        textureNumLevels(depth_pyramid) - 1u
    );
    // Each level halves the one below, rounded down, as a mip chain does:
    // computed, since lavapipe (Mesa 25.2.8) answers a size query whose
    // level varies across a workgroup's invocations with one invocation's.
    let size = max(level_zero_texels >> vec2<u32>(level), vec2<u32>(1u));
    let first = min(vec2<u32>(first_coordinates * vec2<f32>(size)), size - 1u);
    let last = min(vec2<u32>(last_coordinates * vec2<f32>(size)), size - 1u);
    if any(last > first + 1u) {
        return false;
    }
    let farthest = min(
        min(textureLoad(depth_pyramid, first, level).r, textureLoad(depth_pyramid, vec2<u32>(last.x, first.y), level).r),
        min(textureLoad(depth_pyramid, vec2<u32>(first.x, last.y), level).r, textureLoad(depth_pyramid, last, level).r)
    );
    return nearest_depth < farthest;
}

// Whether the Instance in `slot` was visible in the frame before, as phase
// two recorded it: one bit per slot.
fn visible_before(slot: u32) -> bool {
    return extractBits(instance_visibility[slot / 32u], slot % 32u, 1u) == 1u;
}

// A workgroup per block of SCAN_BLOCK_SIZE slots: each slot's Meshlet
// records and packed indices, its Instance's where the phase draws it or
// none, summed over the block's earlier slots into its offsets, NOT_DRAWN in
// their place where it is not drawn, and the block's sum after every slot's.
// Phase one draws an Instance in view that was visible the frame before.
// Phase two finds each Instance in view visible unless it lies behind what
// phase one drew, records which are, and draws those phase one did not.
@compute @workgroup_size(SCAN_BLOCK_SIZE)
fn offset_slots(
    @builtin(workgroup_id) block: vec3<u32>,
    @builtin(local_invocation_index) invocation: u32
) {
    let slot = block.x * SCAN_BLOCK_SIZE + invocation;
    var visible = false;
    var drawn = false;
    var counts = vec2<u32>(0u);
    if slot < instance_slot_count && instances[slot].mesh != NO_MESH {
        let instance = instances[slot];
        let mesh = meshes[instance.mesh];
        let bounds = clip_space_bounds(instance, mesh);
        visible = !CULLING || in_view_frustum(bounds);
        if PHASE_TWO {
            // Behind what phase one drew, it is not visible.
            visible = visible && !(CULLING && occluded(bounds));
            // Phase one's offsets for the slot, before this phase's replace them.
            drawn = CULLING && visible && all(instance_offsets[slot] == NOT_DRAWN);
        } else {
            drawn = visible && (!CULLING || visible_before(slot));
        }
        // A shadow proxy's Mesh has no Meshlets: rays trace it, no pass
        // draws it.
        drawn = drawn && mesh.meshlet_count != 0u;
        if drawn {
            counts = vec2<u32>(mesh.meshlet_count, 3u * mesh.triangle_count);
        }
    }
    if PHASE_TWO {
        if visible {
            atomicOr(&visible_words[invocation / 32u], 1u << (invocation % 32u));
        }
    }
    let offset = exclusive_scan(invocation, counts);
    if PHASE_TWO && invocation < SCAN_BLOCK_SIZE / 32u {
        instance_visibility[block.x * (SCAN_BLOCK_SIZE / 32u) + invocation] =
            atomicLoad(&visible_words[invocation]);
    }
    if slot < instance_slot_count {
        instance_offsets[slot] = select(NOT_DRAWN, offset, drawn);
    }
    if invocation == SCAN_BLOCK_SIZE - 1u {
        instance_offsets[INSTANCE_CAPACITY + block.x] = saturating_add(offset, counts);
    }
}

// One workgroup: each block's offsets, the sum of the earlier blocks' after
// where the phase starts, in place of its own sum; the phase's draw is of
// one Instance; and append_triangles takes a workgroup per Meshlet record
// the phase may append short of the buffer's end, in rows of at most
// MAX_WORKGROUPS_PER_DIMENSION. Each invocation takes a run of blocks, its
// loops bounded alike in every invocation so the scan's barriers stay in
// uniform control flow. Phase one's sums are where phase two starts.
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
    // Phase one's last invocation writes these below: phase one reads none.
    var first = vec2<u32>(0u);
    if PHASE_TWO {
        first = frame_draw.phase_two_offsets;
    }
    var offset = saturating_add(first, exclusive_scan(invocation, sum));
    for (var step = 0u; step < blocks_per_invocation; step++) {
        let block = INSTANCE_CAPACITY + first_block + step;
        if first_block + step < block_count {
            let block_sum = instance_offsets[block];
            instance_offsets[block] = offset;
            offset = saturating_add(offset, block_sum);
        }
    }
    if invocation == SCAN_BLOCK_SIZE - 1u {
        if !PHASE_TWO {
            frame_draw.phase_two_offsets = offset;
        }
        let record_capacity = arrayLength(&meshlet_records);
        frame_draw.records_end = min(offset.x, record_capacity);
        let records = frame_draw.records_end - min(first.x, record_capacity);
        let rows = (records + MAX_WORKGROUPS_PER_DIMENSION - 1u) / MAX_WORKGROUPS_PER_DIMENSION;
        frame_draw.append_dispatch = DispatchIndirect((records + rows - 1u) / max(rows, 1u), rows, 1u);
    }
    if invocation == 0u {
        if PHASE_TWO {
            frame_draw.phase_two_draw.instance_count = 1u;
            frame_draw.phase_two_draw.first_index = min(first.y, arrayLength(&packed_indices));
        } else {
            frame_draw.draw.instance_count = 1u;
        }
    }
}

// The first Meshlet record and packed index of the Instance in `slot`: its
// block's offsets plus its own within the block.
fn slot_offsets(slot: u32) -> vec2<u32> {
    return saturating_add(instance_offsets[INSTANCE_CAPACITY + slot / SCAN_BLOCK_SIZE], instance_offsets[slot]);
}

// Whether the Meshlet records and packed indices of `mesh` from `first` fit
// in their buffers. Where an Instance's would pass either buffer's end,
// neither it nor any Instance after it is drawn, so the draw ends at the
// last Instance that fits, and every index before is written.
fn fits(first: vec2<u32>, mesh: MeshData) -> bool {
    let counts = vec2<u32>(mesh.meshlet_count, 3u * mesh.triangle_count);
    let capacity = vec2<u32>(arrayLength(&meshlet_records), arrayLength(&packed_indices));
    return all(first <= capacity) && all(counts <= capacity - first);
}

// An invocation per Instance slot: writes the Meshlet records of the
// Instance from its offsets, unless they are NOT_DRAWN, those short of the
// buffer's end even where the Instance does not fit, so that every record
// append_triangles reads is written; and counts the Instance drawn, or
// overflowed.
@compute @workgroup_size(SCAN_BLOCK_SIZE)
fn append_records(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let slot = invocation.x;
    if slot >= instance_slot_count || all(instance_offsets[slot] == NOT_DRAWN) {
        return;
    }
    let instance = instances[slot];
    let mesh = meshes[instance.mesh];
    let first = slot_offsets(slot);
    if fits(first, mesh) {
        let indices_end = first.y + 3u * mesh.triangle_count;
        atomicMax(&frame_draw.draw.index_count, indices_end);
        atomicAdd(&frame_draw.instances_drawn, 1u);
        if PHASE_TWO {
            let phase_two_first = frame_draw.phase_two_draw.first_index;
            atomicMax(&frame_draw.phase_two_draw.index_count, indices_end - phase_two_first);
        }
    } else {
        atomicAdd(&frame_draw.instances_overflowed, 1u);
    }
    let overridden = instance.material_id != NO_MATERIAL;
    let records = min(mesh.meshlet_count, arrayLength(&meshlet_records) - min(first.x, arrayLength(&meshlet_records)));
    for (var meshlet_in_mesh = 0u; meshlet_in_mesh < records; meshlet_in_mesh++) {
        let meshlet_index = mesh.first_meshlet + meshlet_in_mesh;
        let material_id = select(meshlets[meshlet_index].material_id, instance.material_id, overridden);
        meshlet_records[first.x + meshlet_in_mesh] = MeshletRecord(slot, meshlet_index, material_id);
    }
}

// A workgroup per Meshlet record the phase appends, dispatched indirectly in
// the rows offset_blocks sized, and an invocation per triangle of its
// Meshlet: writes the record's packed indices where its Instance fits, at
// the Instance's first index plus its Meshlet's triangles before it in the
// Mesh.
@compute @workgroup_size(MESHLET_MAX_TRIANGLES)
fn append_triangles(
    @builtin(workgroup_id) workgroup: vec3<u32>,
    @builtin(num_workgroups) workgroups: vec3<u32>,
    @builtin(local_invocation_index) triangle: u32
) {
    var record_index = workgroup.y * workgroups.x + workgroup.x;
    if PHASE_TWO {
        record_index += frame_draw.phase_two_offsets.x;
    }
    if record_index >= frame_draw.records_end {
        return;
    }
    let record = meshlet_records[record_index];
    let meshlet = meshlets[record.meshlet];
    if triangle >= meshlet.triangle_count {
        return;
    }
    let mesh = meshes[instances[record.instance_slot].mesh];
    let first = slot_offsets(record.instance_slot);
    if !fits(first, mesh) {
        return;
    }
    // The Mesh's triangles in cooked order, each with its corners as cooked,
    // so the draw rasterizes as the cooked index stream does.
    let first_index = first.y + 3u * (meshlet.first_triangle - mesh.first_triangle + triangle);
    let corners = meshlet_triangles[meshlet.first_triangle + triangle];
    for (var corner = 0u; corner < 3u; corner++) {
        let vertex = extractBits(corners, 8u * corner, 8u);
        packed_indices[first_index + corner] = pack_index(record_index, vertex);
    }
}

// The culling pass's entry points, after the shared parts: they append the
// Meshlets of the LOD selected for each Instance they draw to the frame's
// draw in slot order, which the main pass draws with one indirect call per
// phase and depth pipeline, and one of both phases for the lit pipeline
// (ADR 0019), Thin Primitives' Meshlets apart from the others (ADR 0027).
// Phase
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

// The `forced_lod` Render switch: the LOD every Instance draws, the coarsest
// its Mesh has where it has none so coarse, or MESH_MAX_LODS where each
// draws the LOD selected for it.
override FORCED_LOD: u32;

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
// depth before it shades; where phase two's Meshlet records, packed indices
// and Thin Primitives' packed indices start, phase one's sums; the phase's
// dispatch of append_triangles; and each phase's draw of its Thin
// Primitives, whose packed indices end the buffer, phase one's last.
struct FrameDraw {
    draw: DrawIndexedIndirect,
    instances_drawn: atomic<u32>,
    instances_overflowed: atomic<u32>,
    phase_two_draw: DrawIndexedIndirect,
    phase_two_offsets: vec3<u32>,
    append_dispatch: DispatchIndirect,
    phase_one_thin_draw: DrawIndexedIndirect,
    phase_two_thin_draw: DrawIndexedIndirect,
}

// Where instance_offsets holds, after each slot's and block's, the phase's
// first Meshlet record and where its records end, short of the buffer's
// end: append_triangles reads them there, as it runs with another buffer in
// frame_draw's binding, so that frame_draw can hold its dispatch.
const APPENDED_RECORDS = INSTANCE_CAPACITY + INSTANCE_CAPACITY / SCAN_BLOCK_SIZE;

// Where instance_offsets holds, after those, the sums of Thin Primitives'
// packed indices the phase starts and ends at.
const THIN_INDICES = APPENDED_RECORDS + 1u;

var<workgroup> scan: array<vec3<u32>, SCAN_BLOCK_SIZE>;

// Phase two's visibility bits for its block of slots, gathered before they
// are written.
var<workgroup> visible_words: array<atomic<u32>, SCAN_BLOCK_SIZE / 32u>;

// The offsets of a slot the pass does not draw, freed or culled: past every
// capacity, as a sum past a u32 saturates to.
const NOT_DRAWN = vec3<u32>(0xffffffffu);

// `sum + addend`, or the most a u32 holds where that would pass it: past
// every capacity, as the sum it stands for is.
fn saturating_add(sum: vec3<u32>, addend: vec3<u32>) -> vec3<u32> {
    return select(sum + addend, vec3<u32>(0xffffffffu), sum > vec3<u32>(0xffffffffu) - addend);
}

// The sum of `value` over the workgroup's invocations before this one, which
// they all call together: a Hillis and Steele scan
// [ref:hillis-steele-partial-sums], read one place back.
fn exclusive_scan(invocation: u32, value: vec3<u32>) -> vec3<u32> {
    scan[invocation] = value;
    for (var stride = 1u; stride < SCAN_BLOCK_SIZE; stride *= 2u) {
        workgroupBarrier();
        var before = vec3<u32>(0u);
        if invocation >= stride {
            before = scan[invocation - stride];
        }
        workgroupBarrier();
        scan[invocation] = saturating_add(scan[invocation], before);
    }
    workgroupBarrier();
    var before = vec3<u32>(0u);
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

// An Instance's Mesh bounds in clip space, grown to hold a Thin Primitive's
// widening where the Mesh has one: the box's centre, composed relative to
// the camera as its vertices are (ADR 0014), and its half axes, each a
// direction.
struct ClipBox {
    centre: vec4<f32>,
    half_axes: mat3x4<f32>,
}

// The centre of an Instance's Mesh bounds, relative to the camera.
fn bounds_centre(instance: InstanceData, bounds_min: vec3<f32>, bounds_max: vec3<f32>) -> vec3<f32> {
    return camera_relative_position(
        instance.anchor,
        instance.offset,
        camera.anchor,
        camera.offset,
        instance.rotation_scale * (0.5 * (bounds_min + bounds_max))
    );
}

// A Thin Primitive's widening moves a vertex out of its Mesh bounds by less
// than the radius it draws at the box's farthest corner. Lengthening each
// half axis by it holds every vertex only while the axes are orthogonal.
fn clip_space_bounds(instance: InstanceData, mesh: MeshData) -> ClipBox {
    let bounds_min = vec3<f32>(mesh.bounds_min[0], mesh.bounds_min[1], mesh.bounds_min[2]);
    let bounds_max = vec3<f32>(mesh.bounds_max[0], mesh.bounds_max[1], mesh.bounds_max[2]);
    let half_extent = 0.5 * (bounds_max - bounds_min);
    let centre = camera.view_proj * vec4<f32>(bounds_centre(instance, bounds_min, bounds_max), 1.0);
    let half_axes = mat3x3<f32>(
        instance.rotation_scale[0] * half_extent.x,
        instance.rotation_scale[1] * half_extent.y,
        instance.rotation_scale[2] * half_extent.z,
    );
    let clip_half_axes = camera.view_proj * mat3x4<f32>(
        vec4<f32>(half_axes[0], 0.0),
        vec4<f32>(half_axes[1], 0.0),
        vec4<f32>(half_axes[2], 0.0),
    );
    let farthest_w = centre.w
        + abs(clip_half_axes[0].w)
        + abs(clip_half_axes[1].w)
        + abs(clip_half_axes[2].w);
    let widening_metres = select(
        0.0,
        thin_drawn_radius(0.0, camera.pixel_size_per_metre * max(farthest_w, 0.0)),
        mesh.thin_triangle_counts[0] != 0u
    );
    let half_lengths = vec3<f32>(length(half_axes[0]), length(half_axes[1]), length(half_axes[2]));
    let growth = 1.0 + widening_metres / max(half_lengths, vec3<f32>(1e-30));
    return ClipBox(
        centre,
        mat3x4<f32>(
            clip_half_axes[0] * growth.x,
            clip_half_axes[1] * growth.y,
            clip_half_axes[2] * growth.z,
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

// The LOD an Instance draws of its Mesh: the one forced, or the one selected
// at the distance to the sphere about its Mesh bounds, under the Instance's
// largest scale along an axis, so no nearer point of it and no axis strays
// further than the LOD threshold allows: a pixel's size at that distance
// against the error, as meshoptimizer selects a LOD
// [ref:meshoptimizer-0.25-lod-selection]. It reads the camera and the
// Instance alone, so both phases draw an Instance's same LOD.
fn instance_lod(instance: InstanceData, mesh: MeshData) -> u32 {
    if FORCED_LOD < MESH_MAX_LODS {
        return min(FORCED_LOD, max(mesh.lod_count, 1u) - 1u);
    }
    let bounds_min = vec3<f32>(mesh.bounds_min[0], mesh.bounds_min[1], mesh.bounds_min[2]);
    let bounds_max = vec3<f32>(mesh.bounds_max[0], mesh.bounds_max[1], mesh.bounds_max[2]);
    let scale = max(
        max(length(instance.rotation_scale[0]), length(instance.rotation_scale[1])),
        length(instance.rotation_scale[2])
    );
    let radius = 0.5 * length(bounds_max - bounds_min) * scale;
    let distance_metres = max(length(bounds_centre(instance, bounds_min, bounds_max)) - radius, 0.0);
    return selected_lod(
        mesh.lod_count,
        mesh.lod_errors_metres,
        scale,
        distance_metres,
        camera.lod_error_per_metre
    );
}

// The Meshlet records, the packed indices and the Thin Primitives' packed
// indices that LOD `lod` of the Mesh at `mesh_index` in the Mesh table takes
// in the frame's draw.
fn lod_counts(mesh_index: u32, lod: u32) -> vec3<u32> {
    let thin_triangles = meshes[mesh_index].thin_triangle_counts[lod];
    return vec3<u32>(
        meshes[mesh_index].meshlet_counts[lod],
        3u * (meshes[mesh_index].triangle_counts[lod] - thin_triangles),
        3u * thin_triangles
    );
}

// A drawn slot's offsets hold its Instance's LOD above SLOT_LOD_SHIFT bits
// of its first Meshlet record within its block: an offset those bits cannot
// hold is stored as their most, which is past the draw's records as the
// offset is. No LOD fills the bits above, so no drawn slot reads NOT_DRAWN.
const SLOT_RECORD_MASK = (1u << SLOT_LOD_SHIFT) - 1u;

// The LOD the Instance in `slot`, which the phase draws, draws.
fn slot_lod(slot: u32) -> u32 {
    return instance_offsets[slot].x >> SLOT_LOD_SHIFT;
}

// Whether the Instance in `slot` was visible in the frame before, as phase
// two recorded it: one bit per slot.
fn visible_before(slot: u32) -> bool {
    return extractBits(instance_visibility[slot / 32u], slot % 32u, 1u) == 1u;
}

// A workgroup per block of SCAN_BLOCK_SIZE slots: each slot's Meshlet
// records, packed indices and Thin Primitives' packed indices, those of the
// LOD its Instance draws where the phase draws it or none, summed over the
// block's earlier slots into its offsets, under that LOD, NOT_DRAWN in
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
    var counts = vec3<u32>(0u);
    var lod = 0u;
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
        drawn = drawn && mesh.meshlet_counts[0] != 0u;
        if drawn {
            lod = instance_lod(instance, mesh);
            counts = lod_counts(instance.mesh, lod);
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
        let record_and_lod = min(offset.x, SLOT_RECORD_MASK) | (lod << SLOT_LOD_SHIFT);
        instance_offsets[slot] = select(NOT_DRAWN, vec3<u32>(record_and_lod, offset.yz), drawn);
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
// uniform control flow. Phase one's sums are where phase two starts. The
// phase's Thin Primitives' packed indices fill the buffer's end back from
// the phases' before, so each phase's are in slot order.
@compute @workgroup_size(SCAN_BLOCK_SIZE)
fn offset_blocks(@builtin(local_invocation_index) invocation: u32) {
    let block_count = (instance_slot_count + SCAN_BLOCK_SIZE - 1u) / SCAN_BLOCK_SIZE;
    let blocks_per_invocation = (block_count + SCAN_BLOCK_SIZE - 1u) / SCAN_BLOCK_SIZE;
    let first_block = invocation * blocks_per_invocation;
    var sum = vec3<u32>(0u);
    for (var step = 0u; step < blocks_per_invocation; step++) {
        if first_block + step < block_count {
            sum = saturating_add(sum, instance_offsets[INSTANCE_CAPACITY + first_block + step]);
        }
    }
    // Phase one's last invocation writes these below: phase one reads none.
    var first = vec3<u32>(0u);
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
        let appended = min(vec2<u32>(first.x, offset.x), vec2<u32>(record_capacity));
        instance_offsets[APPENDED_RECORDS] = vec3<u32>(appended, 0u);
        let records = appended.y - appended.x;
        // At least one row, so no records dispatch no workgroups.
        let rows = max((records + MAX_WORKGROUPS_PER_DIMENSION - 1u) / MAX_WORKGROUPS_PER_DIMENSION, 1u);
        frame_draw.append_dispatch = DispatchIndirect((records + rows - 1u) / rows, rows, 1u);
        instance_offsets[THIN_INDICES] = vec3<u32>(first.z, offset.z, 0u);
        let index_capacity = arrayLength(&packed_indices);
        let first_thin_index = index_capacity - min(offset.z, index_capacity);
        if PHASE_TWO {
            frame_draw.phase_two_thin_draw.instance_count = 1u;
            frame_draw.phase_two_thin_draw.first_index = first_thin_index;
        } else {
            frame_draw.phase_one_thin_draw.instance_count = 1u;
            frame_draw.phase_one_thin_draw.first_index = first_thin_index;
        }
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

// The first Meshlet record, packed index and Thin Primitives' packed index
// of the Instance in `slot`: its block's offsets plus its own within the
// block.
fn slot_offsets(slot: u32) -> vec3<u32> {
    let in_block = instance_offsets[slot];
    return saturating_add(
        instance_offsets[INSTANCE_CAPACITY + slot / SCAN_BLOCK_SIZE],
        vec3<u32>(in_block.x & SLOT_RECORD_MASK, in_block.yz)
    );
}

// Whether an Instance's `counts` of Meshlet records and packed indices from
// `first` fit in their buffers, short of the Thin Primitives' packed indices
// of the phases so far, which end the packed indices' buffer. Where an
// Instance's would pass either end, or those do not fit the buffer, neither
// it nor any Instance after it is drawn, so each draw ends at the last
// Instance that fits, and every index before is written.
fn fits(first: vec3<u32>, counts: vec3<u32>) -> bool {
    let thin_indices_end = instance_offsets[THIN_INDICES].y;
    let index_capacity = arrayLength(&packed_indices);
    if thin_indices_end > index_capacity {
        return false;
    }
    let capacity = vec2<u32>(arrayLength(&meshlet_records), index_capacity - thin_indices_end);
    return all(first.xy <= capacity) && all(counts.xy <= capacity - first.xy);
}

// Where the buffer holds the Thin Primitives' packed indices of an Instance
// that fits, whose first is the sum `first_thin_index`: the phase's fill the
// buffer's end back from the phases' before.
fn thin_index(first_thin_index: u32) -> u32 {
    let phase = instance_offsets[THIN_INDICES].xy;
    return arrayLength(&packed_indices) - phase.y + (first_thin_index - phase.x);
}

// An invocation per Instance slot: writes the Meshlet records of the
// Instance's LOD from its offsets, unless they are NOT_DRAWN, those short of
// the buffer's end even where the Instance does not fit, so that every record
// append_triangles reads is written; and counts the Instance drawn, or
// overflowed.
@compute @workgroup_size(SCAN_BLOCK_SIZE)
fn append_records(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let slot = invocation.x;
    if slot >= instance_slot_count || all(instance_offsets[slot] == NOT_DRAWN) {
        return;
    }
    let instance = instances[slot];
    let lod = slot_lod(slot);
    let counts = lod_counts(instance.mesh, lod);
    let first = slot_offsets(slot);
    if fits(first, counts) {
        let indices_end = first.y + counts.y;
        atomicMax(&frame_draw.draw.index_count, indices_end);
        atomicAdd(&frame_draw.instances_drawn, 1u);
        let thin_indices_end = first.z + counts.z - instance_offsets[THIN_INDICES].x;
        if PHASE_TWO {
            let phase_two_first = frame_draw.phase_two_draw.first_index;
            atomicMax(&frame_draw.phase_two_draw.index_count, indices_end - phase_two_first);
            atomicMax(&frame_draw.phase_two_thin_draw.index_count, thin_indices_end);
        } else {
            atomicMax(&frame_draw.phase_one_thin_draw.index_count, thin_indices_end);
        }
    } else {
        atomicAdd(&frame_draw.instances_overflowed, 1u);
    }
    let overridden = instance.material_id != NO_MATERIAL;
    let records = min(counts.x, arrayLength(&meshlet_records) - min(first.x, arrayLength(&meshlet_records)));
    let first_meshlet = meshes[instance.mesh].first_meshlets[lod];
    for (var meshlet_in_lod = 0u; meshlet_in_lod < records; meshlet_in_lod++) {
        let meshlet_index = first_meshlet + meshlet_in_lod;
        let material_id = select(meshlets[meshlet_index].material_id, instance.material_id, overridden);
        meshlet_records[first.x + meshlet_in_lod] = MeshletRecord(slot, meshlet_index, material_id);
    }
}

// Invocations of append_triangles, which share a Meshlet's indices: a
// SIMD-group on Apple GPUs, where wider workgroups wrote a dense scene's
// draw slower.
const APPEND_WORKGROUP_SIZE = 32u;

// A workgroup per Meshlet record the phase appends, dispatched indirectly in
// the rows offset_blocks sized: writes the record's packed indices where its
// Instance fits, at the Instance's first index plus its Meshlet's triangles
// before it in the LOD, or for a Thin Primitive's Meshlet at the Instance's
// first of theirs plus the LOD's triangles of theirs before it, an
// invocation per index in turn, so that adjacent invocations write adjacent
// indices.
@compute @workgroup_size(APPEND_WORKGROUP_SIZE)
fn append_triangles(
    @builtin(workgroup_id) workgroup: vec3<u32>,
    @builtin(num_workgroups) workgroups: vec3<u32>,
    @builtin(local_invocation_index) invocation: u32
) {
    let appended = instance_offsets[APPENDED_RECORDS].xy;
    let record_index = appended.x + workgroup.y * workgroups.x + workgroup.x;
    if record_index >= appended.y {
        return;
    }
    let record = meshlet_records[record_index];
    let meshlet = meshlets[record.meshlet];
    let mesh_index = instances[record.instance_slot].mesh;
    let lod = slot_lod(record.instance_slot);
    let first = slot_offsets(record.instance_slot);
    if !fits(first, lod_counts(mesh_index, lod)) {
        return;
    }
    // The LOD's triangles in cooked order, each with its corners as cooked,
    // so the draw rasterizes as the cooked index stream does.
    let triangles_before = meshlet.first_triangle - meshes[mesh_index].first_triangles[lod];
    var first_index = first.y + 3u * triangles_before;
    if meshlet.thin_radius_metres > 0.0 {
        let ordinary_triangles = meshes[mesh_index].triangle_counts[lod]
            - meshes[mesh_index].thin_triangle_counts[lod];
        first_index = thin_index(first.z) + 3u * (triangles_before - ordinary_triangles);
    }
    for (var index = invocation; index < 3u * meshlet.triangle_count; index += APPEND_WORKGROUP_SIZE) {
        let corners = meshlet_triangles[meshlet.first_triangle + index / 3u];
        let vertex = extractBits(corners, 8u * (index % 3u), 8u);
        packed_indices[first_index + index] = pack_index(record_index, vertex);
    }
}

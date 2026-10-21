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

// The frame's draw, cleared before phase one: the draw of the Instances
// appended so far, from index 0, so phase one's, which the depth pre-pass
// draws, then both phases', which the lit pass draws; the Instances it holds
// and those it has no room for, which Frame stats read back; phase two's own
// draw, of the indices after phase one's, which the lit pass draws into
// depth before it shades; and where phase two's Meshlet records and packed
// indices start, phase one's sums.
struct FrameDraw {
    draw: DrawIndexedIndirect,
    instances_drawn: atomic<u32>,
    instances_overflowed: atomic<u32>,
    phase_two_draw: DrawIndexedIndirect,
    phase_two_offsets: vec2<u32>,
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
    let level_zero_size = vec2<f32>(textureDimensions(depth_pyramid, 0));
    let pixel_margin = 0.5 / level_zero_size;
    let first_coordinates = saturate(vec2<f32>(0.5 * low.x + 0.5, 0.5 - 0.5 * high.y) - pixel_margin);
    let last_coordinates = saturate(vec2<f32>(0.5 * high.x + 0.5, 0.5 - 0.5 * low.y) + pixel_margin);
    let extent = (last_coordinates - first_coordinates) * level_zero_size;
    let level = min(
        u32(ceil(log2(max(max(extent.x, extent.y), 1.0)))),
        textureNumLevels(depth_pyramid) - 1u
    );
    let size = textureDimensions(depth_pyramid, level);
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
// where the phase starts, in place of its own sum; and the phase's draw is
// of one Instance. Each invocation takes a run of blocks, its loops bounded
// alike in every invocation so the scan's barriers stay in uniform control
// flow. Phase one's sums are where phase two starts.
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
    if !PHASE_TWO && invocation == SCAN_BLOCK_SIZE - 1u {
        frame_draw.phase_two_offsets = offset;
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
        if triangle == 0u {
            atomicAdd(&frame_draw.instances_overflowed, 1u);
        }
        return;
    }
    if triangle == 0u {
        atomicMax(&frame_draw.draw.index_count, first.y + counts.y);
        atomicAdd(&frame_draw.instances_drawn, 1u);
        if PHASE_TWO {
            let phase_two_first = frame_draw.phase_two_draw.first_index;
            atomicMax(&frame_draw.phase_two_draw.index_count, first.y + counts.y - phase_two_first);
        }
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

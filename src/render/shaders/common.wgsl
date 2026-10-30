// Shared by every shader module: the structs the CPU writes, the Anchor
// composition, the position a depth target's depth places at a pixel, the
// packed index and the vertex decode (src/render/shaders/mod.rs).

// Positions reach the shaders relative to the camera (ADR 0014), so the
// camera sits at the origin and view_proj holds no translation. The camera's
// World position arrives as the anchor of its grid cell plus an offset, its
// exposure as the scale the lit pass applies to luminance (ADR 0023), the
// static TLAS's frame as the Anchor whose corner is its origin, which rays
// start from (ADR 0020), how many entries the dynamic TLAS holds, and a box
// in that frame, its lowest corner then its highest, that holds every
// triangle of theirs.
struct CameraUniform {
    view_proj: mat4x4<f32>,
    anchor: vec3<i32>,
    exposure: f32,
    offset: vec3<f32>,
    tlas_anchor: vec3<i32>,
    dynamic_tlas_entries: u32,
    dynamic_tlas_bounds_min: vec3<f32>,
    dynamic_tlas_bounds_max: vec3<f32>,
}

// ANCHOR_CELL_SIZE, the anchor grid's edge in metres, is declared from
// Rust's when a module is assembled (src/render/shaders/mod.rs).

// A point's position relative to the camera, from its Instance's anchor and
// offset, the camera's, and its position relative to the Instance's pivot
// (ADR 0014). The anchor difference is exact in integers, so f32 only ever
// holds magnitudes near the camera.
fn camera_relative_position(
    anchor: vec3<i32>,
    offset: vec3<f32>,
    camera_anchor: vec3<i32>,
    camera_offset: vec3<f32>,
    local_position: vec3<f32>
) -> vec3<f32> {
    let anchor_position = vec3<f32>(anchor - camera_anchor) * ANCHOR_CELL_SIZE;
    return anchor_position + (offset - camera_offset) + local_position;
}

// The position relative to the camera of the surface a depth target shows
// at `depth` through `point`, in pixels from the top left corner of a target
// of `size`, seen through `view_proj`. view_proj holds no translation (ADR
// 0014), and its reverse-Z projection with no far plane (ADR 0015) puts the
// near plane's distance in clip z and the depth along the view axis in clip
// w, so depth is the near distance over clip w. Its rows are the view's
// axes, x and y scaled by the projection, w along the view axis.
fn position_at(view_proj: mat4x4<f32>, point: vec2<f32>, depth: f32, size: vec2<f32>) -> vec3<f32> {
    let device = vec2<f32>(point.x / size.x * 2.0 - 1.0, 1.0 - point.y / size.y * 2.0);
    let rows = transpose(view_proj);
    let view_depth = view_proj[3].z / depth;
    let right = rows[0].xyz / dot(rows[0].xyz, rows[0].xyz);
    let up = rows[1].xyz / dot(rows[1].xyz, rows[1].xyz);
    return view_depth * (rows[3].xyz + device.x * right + device.y * up);
}

// Whether the segment from `start_distance` to `end_distance` along the ray
// from `origin` along `direction`, a unit vector, meets the box from
// `box_min` to `box_max`, its faces included. No axis may separate them:
// neither the box's three, on which the segment's half-length reach and the
// box's half-extent must cover the distance between their centres, nor the
// three crossings of those with the segment, on which the segment has no
// reach. It divides by nothing, so a ray parallel to a pair of faces needs
// no case of its own. No point of the box lies farther along the ray than
// the origin's distance from the box's centre plus its half-diagonal, so the
// segment is cut there first, which keeps a directional light's, which runs
// to f32's largest distance, and the products below finite.
fn segment_meets_box(
    origin: vec3<f32>,
    direction: vec3<f32>,
    start_distance: f32,
    end_distance: f32,
    box_min: vec3<f32>,
    box_max: vec3<f32>
) -> bool {
    let centre = 0.5 * (box_min + box_max);
    let half_extent = 0.5 * (box_max - box_min);
    let from_centre = origin - centre;
    let cut_end_distance = min(end_distance, length(from_centre) + length(half_extent));
    if cut_end_distance < start_distance {
        return false;
    }
    let half_length = 0.5 * (cut_end_distance - start_distance);
    let midpoint = from_centre + direction * (start_distance + half_length);
    let half_segment = direction * half_length;
    let reach = abs(half_segment);
    if any(abs(midpoint) > half_extent + reach) {
        return false;
    }
    let crossed_reach = vec3<f32>(
        half_extent.y * reach.z + half_extent.z * reach.y,
        half_extent.x * reach.z + half_extent.z * reach.x,
        half_extent.x * reach.y + half_extent.y * reach.x
    );
    return !any(abs(cross(midpoint, half_segment)) > crossed_reach);
}

// A Froxel's list: how many lights it holds, at most FROXEL_LIGHT_CAPACITY,
// then their indices in the light buffer, in light order
// (src/render/froxel_pass.rs).
struct FroxelList {
    light_count: u32,
    lights: array<u32, FROXEL_LIGHT_CAPACITY>,
}

// What the frame's lights counted, which the CPU reads back
// (src/render/froxel_pass.rs): the lights the Froxel pass's lists had no
// room for, then, with the count_point_lights Render switch on, the pixels
// the light count pass counted and the point lights their Froxels list and
// whose range reaches them, summed.
struct LightCounts {
    froxel_lights_overflowed: atomic<u32>,
    lit_pixels: atomic<u32>,
    listed_point_lights: atomic<u32>,
    reaching_point_lights: atomic<u32>,
}

// A point light's position relative to the camera at `camera_anchor` and
// `camera_offset`, composed from its Anchor and offset (ADR 0014).
fn point_light_position(
    light: LightData,
    camera_anchor: vec3<i32>,
    camera_offset: vec3<f32>
) -> vec3<f32> {
    return camera_relative_position(
        light.anchor,
        light.offset,
        camera_anchor,
        camera_offset,
        vec3<f32>(0.0)
    );
}

struct MaterialData {
    base_color: vec4<f32>,
    // Metallic, roughness and the normal Texture's scale; w is padding.
    metallic_roughness: vec4<f32>,
    emissive: vec4<f32>,
    // A Layer per Channel: albedo, normal, metallic-roughness, occlusion;
    // the emissive Channel's is extra.x.
    layers: vec4<u32>,
    extra: vec4<u32>,
};

// The light buffer's counts, its directional lights first, and the scene's
// ambient illuminance.
struct LightUniform {
    // rgb: the scene's ambient illuminance.
    ambient_illuminance: vec4<f32>,
    directional_light_count: u32,
    point_light_count: u32,
}

// A light as the light buffer holds it, directional lights first, then point
// lights. A point light's position arrives as an Anchor plus an offset
// (ADR 0014), with its range; a directional light has no position, and its
// offset holds the direction it shines along. Its intensity is its colour
// times its candela, or a directional light's times its lux (ADR 0023).
struct LightData {
    anchor: vec3<i32>,
    range: f32,
    offset: vec3<f32>,
    intensity: vec3<f32>,
}

// An Instance's pivot as anchor plus offset, so camera motion changes no
// Instance, and its rotation-scale; its Mesh, or NO_MESH in a freed slot;
// and the Material ID that overrides its Mesh's, or NO_MATERIAL. Both are
// declared from Rust's (src/render/shaders/mod.rs).
struct InstanceData {
    rotation_scale: mat3x3<f32>,
    normal_matrix: mat3x3<f32>,
    anchor: vec3<i32>,
    material_id: u32,
    offset: vec3<f32>,
    mesh: u32,
}

// A Mesh as the Mesh table holds it, at its Handle's index: its Meshlets,
// whose triangles follow one another in the Meshlet triangle Arena, and the
// box that holds its vertices, its lowest corner, then its highest.
struct MeshData {
    first_meshlet: u32,
    meshlet_count: u32,
    first_triangle: u32,
    triangle_count: u32,
    bounds_min: array<f32, 3>,
    bounds_max: array<f32, 3>,
}

// A Meshlet as the Mesh pool's Meshlet Arena holds it: its first vertex in
// the Meshlet vertex Arena, each an index into the vertex Arena; its first
// triangle in the Meshlet triangle Arena, each three corners indexing those
// vertices, a byte each, lowest byte first; and its Primitive's Material ID.
struct MeshletData {
    first_vertex: u32,
    first_triangle: u32,
    triangle_count: u32,
    material_id: u32,
}

// One Meshlet of one Instance in the frame's draw, by the Instance's slot,
// with the Material it is drawn with.
struct MeshletRecord {
    instance_slot: u32,
    meshlet: u32,
    material_id: u32,
}

// An index of the frame's draw, unpacked: the Meshlet record it is drawn
// for, and its vertex within that record's Meshlet (ADR 0019).
struct PackedIndex {
    record: u32,
    vertex: u32,
}

// The record in the bits above MESHLET_VERTEX_BITS, which is declared from
// Rust's Meshlet vertex limit (src/render/shaders/mod.rs), the vertex below.
fn pack_index(record: u32, vertex: u32) -> u32 {
    return (record << MESHLET_VERTEX_BITS) | vertex;
}

fn unpack_index(index: u32) -> PackedIndex {
    return PackedIndex(index >> MESHLET_VERTEX_BITS, index & ((1u << MESHLET_VERTEX_BITS) - 1u));
}

// A vertex as the vertex Arena stores it, 28 bytes (ADR 0018). Arrays of
// scalars keep vector alignment out of its layout, and WGSL has no 16-bit
// integers, so the QTangent's Snorm16 components arrive in u32 pairs.
struct CookedVertex {
    position: array<f32, 3>,
    uv: array<f32, 2>,
    qtangent: array<u32, 2>,
}

// A vertex's normal, tangent and bitangent sign (glTF's convention).
struct TangentFrame {
    normal: vec3<f32>,
    tangent: vec3<f32>,
    bitangent_sign: f32,
}

// A QTangent is the unit quaternion that turns tangent space (tangent,
// bitangent, normal) into the mesh's, its w's sign the bitangent sign; the
// cooker keeps w off zero so the sign survives (ADR 0018). Snorm16 leaves the
// quaternion just off unit length, never near zero.
fn decode_qtangent(qtangent: vec4<f32>) -> TangentFrame {
    let rotation = normalize(qtangent);
    return TangentFrame(
        rotate(rotation, vec3<f32>(0.0, 0.0, 1.0)),
        rotate(rotation, vec3<f32>(1.0, 0.0, 0.0)),
        select(1.0, -1.0, rotation.w < 0.0),
    );
}

// `vector` turned by the unit quaternion `rotation` (q v q*), expanded as
// v + 2 r × (r × v + w v), with r its vector part and w its scalar part.
fn rotate(rotation: vec4<f32>, vector: vec3<f32>) -> vec3<f32> {
    let vector_part = rotation.xyz;
    return vector + 2.0 * cross(vector_part, cross(vector_part, vector) + rotation.w * vector);
}

// A vertex in mesh space, as the vertex shader uses it.
struct Vertex {
    position: vec3<f32>,
    uv: vec2<f32>,
    tangent_frame: TangentFrame,
}

// `vertex`'s QTangent, unpacked from Snorm16: each component over 32,767,
// clamped at −1.
fn unpack_qtangent(vertex: CookedVertex) -> vec4<f32> {
    return vec4<f32>(unpack2x16snorm(vertex.qtangent[0]), unpack2x16snorm(vertex.qtangent[1]));
}

// `vertex` decoded.
fn decode_vertex(vertex: CookedVertex) -> Vertex {
    let qtangent = unpack_qtangent(vertex);
    return Vertex(
        vec3<f32>(vertex.position[0], vertex.position[1], vertex.position[2]),
        vec2<f32>(vertex.uv[0], vertex.uv[1]),
        decode_qtangent(qtangent),
    );
}

// A unit vector's octahedral coordinates, each in [-1, 1]: it projects onto
// the octahedron |x| + |y| + |z| = 1, whose lower half folds over the upper.
fn octahedral_encode(direction: vec3<f32>) -> vec2<f32> {
    let projected = direction.xy / (abs(direction.x) + abs(direction.y) + abs(direction.z));
    let sign = select(vec2<f32>(-1.0), vec2<f32>(1.0), projected >= vec2<f32>(0.0));
    return select((1.0 - abs(projected.yx)) * sign, projected, direction.z >= 0.0);
}

// The unit vector whose octahedral coordinates are `coordinates`.
fn octahedral_decode(coordinates: vec2<f32>) -> vec3<f32> {
    let z = 1.0 - abs(coordinates.x) - abs(coordinates.y);
    let sign = select(vec2<f32>(-1.0), vec2<f32>(1.0), coordinates >= vec2<f32>(0.0));
    let unfolded = select((1.0 - abs(coordinates.yx)) * sign, coordinates, z >= 0.0);
    return normalize(vec3<f32>(unfolded, z));
}

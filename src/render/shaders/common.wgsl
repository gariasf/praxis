// Shared by every shader module: the structs the CPU writes, the Anchor
// composition and the vertex decode (src/render/shaders/mod.rs).

// Positions reach the shaders relative to the camera (ADR 0014), so the
// camera sits at the origin and view_proj holds no translation. The camera's
// World position arrives as the anchor of its grid cell plus an offset.
struct CameraUniform {
    view_proj: mat4x4<f32>,
    anchor: vec3<i32>,
    offset: vec3<f32>,
}

// Edge of the anchor grid in metres; ANCHOR_CELL_SIZE in Rust matches it.
const ANCHOR_CELL_SIZE: f32 = 64.0;

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

struct MaterialData {
    base_color: vec4<f32>,
    metallic_roughness: vec4<f32>,
    emissive: vec4<f32>,
    texture_indices: vec4<u32>,
    extra: vec4<u32>,
};

struct LightUniform {
    direction: vec4<f32>,
    color: vec4<f32>,
    ambient: vec4<f32>,

    // Point light
    point_positions: array<vec4<f32>, 4>,
    point_colors: array<vec4<f32>, 4>,
    // x = point lights, y = directional lights
    light_counts: vec4<u32>,
}

// An Instance's pivot as anchor plus offset, so camera motion changes no
// Instance, and its rotation-scale.
struct InstanceData {
    rotation_scale: mat3x3<f32>,
    normal_matrix: mat3x3<f32>,
    anchor: vec3<i32>,
    material_id: u32,
    offset: vec3<f32>,
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

// `vertex` decoded, its QTangent unpacked from Snorm16 (each component over
// 32,767, clamped at −1).
fn decode_vertex(vertex: CookedVertex) -> Vertex {
    let qtangent = vec4<f32>(
        unpack2x16snorm(vertex.qtangent[0]),
        unpack2x16snorm(vertex.qtangent[1])
    );
    return Vertex(
        vec3<f32>(vertex.position[0], vertex.position[1], vertex.position[2]),
        vec2<f32>(vertex.uv[0], vertex.uv[1]),
        decode_qtangent(qtangent),
    );
}

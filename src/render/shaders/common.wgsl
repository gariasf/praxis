// Shared by every shader module: the structs the CPU writes, the Anchor
// composition and the QTangent decode (src/render/shaders/mod.rs).

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
    num_point_lights: vec4<f32>,
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

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    // Snorm16x4, decoded by decode_qtangent.
    @location(2) qtangent: vec4<f32>,
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

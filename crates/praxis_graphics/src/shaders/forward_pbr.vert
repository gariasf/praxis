#version 450

// Forward rendering vertex shader with PBR support
// Transforms vertices and prepares data for fragment shader lighting

// Input vertex attributes
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec3 color;
layout(location = 3) in vec2 uv;
layout(location = 4) in vec4 tangent;

// Output to fragment shader
layout(location = 0) out vec3 v_world_pos;
layout(location = 1) out vec3 v_normal;
layout(location = 2) out vec3 v_color;
layout(location = 3) out vec2 v_uv;
layout(location = 4) out vec3 v_tangent;
layout(location = 5) out vec3 v_bitangent;

// Uniforms
layout(set = 0, binding = 0) uniform ViewProjection {
    mat4 view;
    mat4 proj;
    vec3 camera_position;
    float _padding;
} view_proj;

layout(set = 0, binding = 1) uniform Model {
    mat4 model;
} model_ubo;

void main() {
    // Transform position to world space
    vec4 world_pos = model_ubo.model * vec4(position, 1.0);
    v_world_pos = world_pos.xyz;
    
    // Transform to clip space (MVP)
    gl_Position = view_proj.proj * view_proj.view * world_pos;
    
    // Transform normal to world space
    v_normal = mat3(model_ubo.model) * normal;
    
    // Transform tangent to world space
    vec3 world_tangent = mat3(model_ubo.model) * tangent.xyz;
    v_tangent = world_tangent;
    
    // Compute bitangent (perpendicular to normal and tangent)
    v_bitangent = cross(v_normal, world_tangent) * tangent.w;
    
    // Pass through color and UV
    v_color = color;
    v_uv = uv;
}

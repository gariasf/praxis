#version 450

// Vertex shader for deferred rendering geometry pass
// Transforms vertices and passes data to fragment shader for G-buffer output

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec3 color;
layout(location = 3) in vec2 uv;
layout(location = 4) in vec4 tangent;

layout(location = 0) out vec3 v_world_pos;
layout(location = 1) out vec3 v_normal;
layout(location = 2) out vec3 v_color;
layout(location = 3) out vec2 v_uv;

layout(set = 0, binding = 0, std140) uniform ViewProjection {
    mat4 view;
    mat4 proj;
    vec3 camera_position;
    float _padding;
} view_proj;

layout(set = 0, binding = 1, std140) uniform Model {
    mat4 model;
} model_ubo;

void main() {
    vec4 world_pos = model_ubo.model * vec4(position, 1.0);
    gl_Position = view_proj.proj * view_proj.view * world_pos;
    
    v_world_pos = world_pos.xyz;
    v_normal = mat3(model_ubo.model) * normal;
    v_color = color;
    v_uv = uv;
}

#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 v_color;
layout(location = 1) out vec2 v_uv;

layout(set = 0, binding = 0, std140) uniform Uniforms {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;  // UBO = Uniform Buffer Object

void main() {
    gl_Position = ubo.proj * ubo.view * ubo.model * vec4(position, 1.0);
    v_color = color;
    v_uv = uv;
}

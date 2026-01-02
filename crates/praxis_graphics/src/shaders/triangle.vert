#version 450

// Vertex Shader for 3D Textured Rendering
//
// This shader transforms vertices from model space to clip space and passes
// color and UV coordinates to the fragment shader for texture sampling.
//
// Input Layout (from Vertex3D):
//   location 0: vec3 position - vertex position in model space
//   location 1: vec3 color    - vertex color (RGB)
//   location 2: vec2 uv       - texture coordinates
//
// Uniform Buffer (set 0, binding 0):
//   mat4 model - model matrix (model → world)
//   mat4 view  - view matrix (world → view/camera)
//   mat4 proj  - projection matrix (view → clip space)
//
// Output:
//   gl_Position - transformed vertex position in clip space
//   v_color     - color passed to fragment shader
//   v_uv        - UV coordinates passed to fragment shader

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
    // Transform vertex position: model space → world → view → clip space
    gl_Position = ubo.proj * ubo.view * ubo.model * vec4(position, 1.0);
    
    // Pass color to fragment shader
    v_color = color;
    
    // Pass UV coordinates to fragment shader for texture sampling
    v_uv = uv;
}

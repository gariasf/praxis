#version 450

// Vertex Shader for 3D Textured Rendering with Lighting
//
// This shader transforms vertices from model space to clip space and passes
// normals, world position, color and UV coordinates to the fragment shader
// for lighting and texture sampling.
//
// Input Layout (from Vertex3D):
//   location 0: vec3 position - vertex position in model space
//   location 1: vec3 normal   - vertex normal in model space
//   location 2: vec3 color    - vertex color (RGB)
//   location 3: vec2 uv       - texture coordinates
//
// Uniform Buffer (set 0, binding 0):
//   mat4 model - model matrix (model → world)
//   mat4 view  - view matrix (world → view/camera)
//   mat4 proj  - projection matrix (view → clip space)
//
// Output:
//   gl_Position   - transformed vertex position in clip space
//   v_world_pos   - vertex position in world space (for lighting)
//   v_normal      - normal in world space (for lighting)
//   v_color       - color passed to fragment shader
//   v_uv          - UV coordinates passed to fragment shader

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec3 color;
layout(location = 3) in vec2 uv;

layout(location = 0) out vec3 v_world_pos;
layout(location = 1) out vec3 v_normal;
layout(location = 2) out vec3 v_color;
layout(location = 3) out vec2 v_uv;

layout(set = 0, binding = 0, std140) uniform Uniforms {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;  // UBO = Uniform Buffer Object

void main() {
    // Transform vertex position: model space → world → view → clip space
    vec4 world_pos = ubo.model * vec4(position, 1.0);
    gl_Position = ubo.proj * ubo.view * world_pos;
    
    // Pass world position for lighting calculations
    v_world_pos = world_pos.xyz;
    
    // Transform normal to world space (assuming uniform scaling)
    // For non-uniform scaling, use transpose(inverse(mat3(model)))
    v_normal = mat3(ubo.model) * normal;
    
    // Pass color to fragment shader
    v_color = color;
    
    // Pass UV coordinates to fragment shader for texture sampling
    v_uv = uv;
}

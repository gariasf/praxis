#version 450

// ============================================================================
// Shadow Map Vertex Shader
// ============================================================================
//
// This shader is used during the shadow pass to render the scene from the
// light's perspective. It's much simpler than the main vertex shader because
// we only need to output depth values, not colors or lighting data.

// ============================================================================
// Input Vertex Attributes
// ============================================================================

layout(location = 0) in vec3 position;  // Vertex position in model space
layout(location = 1) in vec3 normal;    // Not used in shadow pass
layout(location = 2) in vec3 color;     // Not used in shadow pass
layout(location = 3) in vec2 uv;        // Not used in shadow pass

// ============================================================================
// Per-Object Uniform Buffer (Model Matrix) - DYNAMIC
// ============================================================================

layout(set = 0, binding = 0, std140) uniform Model {
    mat4 model;  // Model matrix: transforms model space → world space
} model_ubo;

// ============================================================================
// Light-Space Uniform Buffer
// ============================================================================

layout(set = 0, binding = 1, std140) uniform LightSpace {
    mat4 light_space_matrix;  // Light view-projection matrix
} light_space;

// ============================================================================
// Main Vertex Shader
// ============================================================================

void main() {
    // Transform vertex from model space to world space, then to light space
    // This is similar to MVP transform, but using the light's view-projection
    vec4 world_pos = model_ubo.model * vec4(position, 1.0);
    gl_Position = light_space.light_space_matrix * world_pos;
}

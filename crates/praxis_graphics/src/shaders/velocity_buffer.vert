#version 450

// Velocity buffer generation vertex shader
// Transforms vertices with both current and previous matrices

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform VelocityUniforms {
    mat4 current_mvp;   // Current frame model-view-projection matrix
    mat4 previous_mvp;  // Previous frame model-view-projection matrix
} uniforms;

layout(location = 0) out vec4 current_pos;
layout(location = 1) out vec4 previous_pos;

void main() {
    // Transform vertex with current and previous matrices
    current_pos = uniforms.current_mvp * vec4(position, 1.0);
    previous_pos = uniforms.previous_mvp * vec4(position, 1.0);
    
    // Output position for rasterization
    gl_Position = current_pos;
}
